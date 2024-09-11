/*
  Copyright 2024 Mike Nguyen (mikeee) <hey@mike.ee>

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/
use core::fmt;
use std::{error::Error, time::SystemTime};

use prost_wkt_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    api,
    durabletask_pb::{
        history_event::EventType, orchestrator_action::OrchestratorActionType,
        ExecutionCompletedEvent, ExecutionStartedEvent, HistoryEvent, OrchestrationStatus,
        OrchestratorAction, SubOrchestrationInstanceCompletedEvent, TaskFailureDetails,
    },
    internal::{self, to_runtime_status_string},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestratorMessage {
    history_event: Option<HistoryEvent>,
    target_instance_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OrchestrationRuntimeState {
    instance_id: api::InstanceID,
    pub(crate) new_events: Vec<HistoryEvent>,
    pub(crate) old_events: Vec<HistoryEvent>,
    pending_tasks: Vec<HistoryEvent>,
    pending_timers: Vec<HistoryEvent>,
    pending_messages: Vec<OrchestratorMessage>,
    start_event: Option<ExecutionStartedEvent>,
    completed_event: Option<ExecutionCompletedEvent>,
    created_time: Option<SystemTime>,
    last_updated_time: Option<SystemTime>,
    completed_time: Option<SystemTime>,
    continued_as_new: bool,
    is_suspended: bool,
    custom_status: Option<String>,
}

impl OrchestrationRuntimeState {
    fn new(instance_id: &api::InstanceID, existing_history: &[HistoryEvent]) -> Self {
        let mut state = OrchestrationRuntimeState {
            instance_id: instance_id.to_owned(),
            new_events: Vec::with_capacity(10),
            old_events: Vec::with_capacity(existing_history.len()),
            ..Default::default()
        };

        for event in existing_history {
            let _ = state.add_event(event, false);
        }

        state
    }

    pub fn add_event(&mut self, event: &HistoryEvent, is_new: bool) -> Result<(), Box<dyn Error>> {
        match &event.event_type {
            Some(EventType::ExecutionStarted(started_event)) => {
                if self.start_event.is_some() {
                    return Err("Duplicate start event".into());
                }
                self.start_event = Some(started_event.clone());
                self.created_time = Some(SystemTime::try_from(event.timestamp.clone().unwrap())?);
            }
            Some(EventType::ExecutionCompleted(completed_event)) => {
                if self.completed_event.is_some() {
                    return Err("Duplicate completed event".into());
                }
                self.completed_event = Some(completed_event.clone());
                self.completed_time = Some(SystemTime::try_from(event.timestamp.clone().unwrap())?);
            }
            Some(EventType::ExecutionSuspended(_)) => {
                self.is_suspended = true;
            }
            Some(EventType::ExecutionResumed(_)) => {
                self.is_suspended = false;
            }
            _ => {
                // TODO: Check for other possible duplicates using task IDs
            }
        }

        self.last_updated_time = Some(SystemTime::try_from(event.timestamp.clone().unwrap())?);

        if is_new {
            self.new_events.push(event.clone());
        } else {
            self.old_events.push(event.clone());
        }

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        (self.old_events.is_empty() && self.new_events.is_empty()) || self.start_event.is_some()
    }

    pub fn apply_actions(
        &mut self,
        actions: &[OrchestratorAction],
    ) -> Result<bool, Box<dyn Error>> {
        let mut continued_as_new = false;

        for action in actions {
            match &action.orchestrator_action_type {
                Some(OrchestratorActionType::CompleteOrchestration(completed_action)) => {
                    if completed_action.orchestration_status()
                        == OrchestrationStatus::ContinuedAsNew
                    {
                        let mut new_state = OrchestrationRuntimeState::new(&self.instance_id, &[]);
                        new_state.continued_as_new = true;

                        let _ = new_state.add_event(
                            &internal::new_execution_started_event(
                                &self.start_event.clone().unwrap().name,
                                &self.instance_id(),
                                completed_action.result.as_deref(),
                                self.start_event.clone().unwrap().parent_instance,
                                self.start_event.clone().unwrap().parent_trace_context,
                                None,
                            ),
                            true,
                        );

                        for event in completed_action.carryover_events.iter() {
                            new_state.add_event(event, true)?;
                        }

                        *self = new_state;
                        continued_as_new = true;
                        return Ok(continued_as_new);
                    } else {
                        self.add_event(
                            &HistoryEvent {
                                event_id: -1,
                                timestamp: Some(Timestamp::from(SystemTime::now())),
                                event_type: Some(EventType::ExecutionCompleted(
                                    ExecutionCompletedEvent {
                                        orchestration_status: completed_action.orchestration_status,
                                        result: completed_action.result.clone(),
                                        failure_details: completed_action.failure_details.clone(),
                                    },
                                )),
                            },
                            true,
                        )?;

                        if let Some(parent_instance) =
                            self.start_event.as_ref().unwrap().parent_instance.as_ref()
                        {
                            self.pending_messages.push(OrchestratorMessage {
                                history_event: Some(HistoryEvent {
                                    event_id: -1,
                                    timestamp: Some(Timestamp::from(SystemTime::now())),
                                    event_type: Some(EventType::SubOrchestrationInstanceCompleted(
                                        SubOrchestrationInstanceCompletedEvent {
                                            task_scheduled_id: parent_instance.task_scheduled_id,
                                            result: completed_action.result.clone(),
                                        },
                                    )),
                                }),
                                target_instance_id: parent_instance
                                    .to_owned()
                                    .orchestration_instance
                                    .unwrap()
                                    .instance_id,
                            });
                        }
                    }
                }
                Some(OrchestratorActionType::CreateTimer(create_timer)) => {
                    self.add_event(
                        &internal::new_timer_created_event(
                            action.id,
                            &create_timer.fire_at.clone().unwrap(),
                        ),
                        true,
                    )?;
                    self.pending_timers.push(internal::new_timer_fired_event(
                        action.id,
                        &create_timer.fire_at.clone().unwrap(),
                    ));
                }
                Some(OrchestratorActionType::ScheduleTask(schedule_task)) => {
                    let scheduled_event = internal::new_task_scheduled_event(
                        action.id,
                        &schedule_task.name,
                        schedule_task.version.as_deref(),
                        schedule_task.input.as_deref(),
                        None, // TODO: Revisit context
                    );

                    self.add_event(&scheduled_event, true)?;
                    self.pending_tasks.push(scheduled_event);
                }
                Some(OrchestratorActionType::CreateSubOrchestration(create_so)) => {
                    let instance_id = if create_so.instance_id.is_empty() {
                        format!("{:?}:{:04x}", self.instance_id, action.id)
                    } else {
                        create_so.instance_id.clone()
                    };

                    let sub_orchestration_created_event =
                        internal::new_sub_orchestration_created_event(
                            action.id,
                            &create_so.name,
                            create_so.version.as_deref(),
                            create_so.input.as_deref(),
                            &create_so.instance_id,
                            None, // TODO: Revisit context
                        );

                    self.add_event(&sub_orchestration_created_event, true)?;

                    let sub_orchestration_start_event = internal::new_execution_started_event(
                        &create_so.name,
                        &create_so.instance_id,
                        create_so.input.as_deref(),
                        Some(internal::new_parent_info(
                            action.id,
                            &self.start_event.as_ref().unwrap().name,
                            &self.instance_id(),
                        )),
                        None, // TODO: Revisit context
                        None,
                    );
                    self.pending_messages.push(OrchestratorMessage {
                        history_event: Some(sub_orchestration_start_event),
                        target_instance_id: instance_id,
                    });
                }
                Some(OrchestratorActionType::SendEvent(send_event)) => {
                    let send_event_event = internal::new_event_sent_event(
                        action.id,
                        &send_event.instance.as_ref().unwrap().instance_id,
                        &send_event.name,
                        send_event.data.as_deref(),
                    );

                    self.add_event(&send_event_event, true)?;
                    self.pending_messages.push(OrchestratorMessage {
                        history_event: Some(send_event_event),
                        target_instance_id: send_event.instance.clone().unwrap().instance_id,
                    });
                }
                Some(OrchestratorActionType::TerminateOrchestration(terminate)) => {
                    let terminate_event = internal::new_execution_terminated_event(
                        terminate.reason.as_deref(),
                        terminate.recurse,
                    );

                    self.pending_messages.push(OrchestratorMessage {
                        history_event: Some(terminate_event),
                        target_instance_id: terminate.instance_id.clone(),
                    });
                }
                _ => {
                    return Err(format!("Unknown action type: {:?}", action).into());
                }
            }
        }

        Ok(continued_as_new)
    }

    pub fn instance_id(&self) -> String {
        self.instance_id.to_owned().0
    }

    pub fn name(&self) -> Result<&str, Box<dyn Error>> {
        if let Some(start_event) = &self.start_event {
            Ok(&start_event.name)
        } else {
            Err(api::ERR_NOT_STARTED.into())
        }
    }

    pub fn input(&self) -> Result<&str, Box<dyn Error>> {
        if let Some(start_event) = &self.start_event {
            Ok(start_event.input.as_ref().unwrap())
        } else {
            Err(api::ERR_NOT_STARTED.into())
        }
    }

    pub fn output(&self) -> Result<&str, Box<dyn Error>> {
        if let Some(completed_event) = &self.completed_event {
            Ok(completed_event.result.as_ref().unwrap())
        } else {
            Err(api::ERR_NOT_COMPLETED.into())
        }
    }

    pub fn runtime_status(&self) -> OrchestrationStatus {
        if self.start_event.is_none() {
            OrchestrationStatus::Pending
        } else if self.is_suspended {
            OrchestrationStatus::Suspended
        } else if let Some(completed_event) = &self.completed_event {
            completed_event.orchestration_status()
        } else {
            OrchestrationStatus::Running
        }
    }

    pub fn created_time(&self) -> Result<SystemTime, Box<dyn Error>> {
        if let Some(_start_event) = &self.start_event {
            Ok(self.created_time.expect("created time"))
        } else {
            Err(api::ERR_NOT_STARTED.into())
        }
    }

    pub fn last_updated_time(&self) -> Result<SystemTime, Box<dyn Error>> {
        if let Some(_start_event) = &self.start_event {
            Ok(self.last_updated_time.expect("last updated time"))
        } else {
            Err(api::ERR_NOT_STARTED.into())
        }
    }

    pub fn completed_time(&self) -> Result<SystemTime, Box<dyn Error>> {
        if let Some(_completed_event) = &self.completed_event {
            Ok(self.completed_time.expect("system time"))
        } else {
            Err(api::ERR_NOT_COMPLETED.into())
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed_event.is_some()
    }

    pub fn old_events(&self) -> &[HistoryEvent] {
        &self.old_events
    }

    pub fn new_events(&self) -> &[HistoryEvent] {
        &self.new_events
    }

    pub fn failure_details(&self) -> Result<&TaskFailureDetails, Box<dyn Error>> {
        if let Some(completed_event) = &self.completed_event {
            if let Some(failure_details) = &completed_event.failure_details {
                Ok(failure_details)
            } else {
                Err(api::ERR_NO_FAILURES.into())
            }
        } else {
            Err(api::ERR_NOT_COMPLETED.into())
        }
    }

    pub fn pending_timers(&self) -> &[HistoryEvent] {
        &self.pending_timers
    }

    pub fn pending_tasks(&self) -> &[HistoryEvent] {
        &self.pending_tasks
    }

    pub fn pending_messages(&self) -> &[OrchestratorMessage] {
        &self.pending_messages
    }

    pub fn continued_as_new(&self) -> bool {
        self.continued_as_new
    }

    #[allow(dead_code)] // TODO: Remove dead_code exception
    pub(crate) fn get_started_time(&self) -> SystemTime {
        if !self.old_events().is_empty() {
            SystemTime::try_from(self.old_events[0].clone().timestamp.unwrap()).unwrap()
        } else if !self.new_events().is_empty() {
            SystemTime::try_from(self.new_events[0].clone().timestamp.unwrap()).unwrap()
        } else {
            SystemTime::UNIX_EPOCH
        }
    }
}

impl fmt::Display for OrchestrationRuntimeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}:{}",
            self.instance_id(),
            to_runtime_status_string(self.runtime_status())
        )
    }
}
