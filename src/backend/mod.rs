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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use prost::Message;

use crate::api::{InstanceID, OrchestrationIdReusePolicy, OrchestrationMetadata};
use crate::backend::runtimestate::OrchestrationRuntimeState;
use crate::backend::workitem::{ActivityWorkItem, OrchestrationWorkItem};
use crate::durabletask_pb::history_event::EventType::SubOrchestrationInstanceCreated;
use crate::durabletask_pb::{ExecutionTerminatedEvent, HistoryEvent};
use crate::internal::new_execution_terminated_event;

pub mod logger;
pub mod orchestration;
pub mod runtimestate;
pub mod workitem;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum BackendError {
    TaskHubExists,
    TaskHubNotFound,
    NotInitialized,
    WorkItemLockLost,
    BackendAlreadyStarted,
    Other(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BackendError::TaskHubExists => write!(f, "task hub already exists"),
            BackendError::TaskHubNotFound => write!(f, "task hub not found"),
            BackendError::NotInitialized => write!(f, "backend not initialized"),
            BackendError::WorkItemLockLost => write!(f, "lock on work-item was lost"),
            BackendError::BackendAlreadyStarted => write!(f, "backend is already started"),
            BackendError::Other(e) => write!(f, "other error: {}", e),
        }
    }
}

impl Error for BackendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BackendError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<Box<dyn Error + Send + Sync>> for BackendError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        BackendError::Other(error)
    }
}

#[allow(dead_code)] // TODO: Remove
pub(crate) type OrchestrationIdReusePolicyOptions =
    Box<dyn Fn(&mut OrchestrationIdReusePolicy) -> Result<(), Box<dyn Error>> + Send + Sync>;

#[allow(dead_code)] // TODO: Remove
pub(crate) fn with_orchestration_id_reuse_policy(
    policy: Option<OrchestrationIdReusePolicy>,
) -> OrchestrationIdReusePolicyOptions {
    Box::new(move |po: &mut OrchestrationIdReusePolicy| {
        if let Some(p) = policy.as_ref() {
            po.action = p.action;
            po.operation_status.clone_from(&p.operation_status);
        }
        Ok(())
    })
}

#[allow(dead_code)] // TODO: Remove
#[async_trait]
pub(crate) trait Backend: Send + Sync {
    async fn create_task_hub(&self) -> Result<(), BackendError>;
    async fn delete_task_hub(&self) -> Result<(), BackendError>;
    async fn start(&self) -> Result<(), BackendError>;
    async fn stop(&self) -> Result<(), BackendError>;
    async fn create_orchestration_instance(
        &self,
        event: &HistoryEvent,
        options: Vec<OrchestrationIdReusePolicyOptions>,
    ) -> Result<(), BackendError>;
    async fn add_new_orchestration_event(
        &self,
        instance_id: &str,
        event: &HistoryEvent,
    ) -> Result<(), BackendError>;
    async fn get_orchestration_work_item(&self) -> Result<OrchestrationWorkItem, BackendError>;
    async fn get_orchestration_runtime_state(
        &self,
        work_item: &OrchestrationWorkItem,
    ) -> Result<OrchestrationRuntimeState, BackendError>;
    async fn get_orchestration_metadata(
        &self,
        instance_id: &str,
    ) -> Result<OrchestrationMetadata, BackendError>;
    async fn complete_orchestration_work_item(
        &self,
        work_item: &OrchestrationWorkItem,
    ) -> Result<(), BackendError>;
    async fn abandon_orchestration_work_item(
        &self,
        work_item: &OrchestrationWorkItem,
    ) -> Result<(), BackendError>;
    async fn get_activity_work_item(&self) -> Result<ActivityWorkItem, BackendError>;
    async fn complete_activity_work_item(
        &self,
        work_item: &ActivityWorkItem,
    ) -> Result<(), BackendError>;
    async fn abandon_activity_work_item(
        &self,
        work_item: &ActivityWorkItem,
    ) -> Result<(), BackendError>;
    async fn purge_orchestration_state(&self, instance_id: &InstanceID)
        -> Result<(), BackendError>;
}

#[allow(dead_code)] // TODO: Remove
pub(crate) fn marshal_history_event(e: &HistoryEvent) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = Vec::new();
    e.encode(&mut buf)?;
    Ok(buf)
}

#[allow(dead_code)] // TODO: Remove
pub(crate) fn unmarshal_history_event(bytes: &[u8]) -> Result<HistoryEvent, Box<dyn Error>> {
    HistoryEvent::decode(bytes).map_err(|e| Box::new(e) as Box<dyn Error>)
}

#[allow(dead_code)] // TODO: Remove
pub(crate) fn purge_orchestration_state<'a>(
    be: &'a (dyn Backend + 'a),
    instance_id: &'a InstanceID,
    recursive: bool,
) -> Pin<Box<dyn Future<Output = Result<i32, BackendError>> + 'a>> {
    Box::pin(async move {
        let mut deleted_instance_count = 0;
        if recursive {
            let owi = OrchestrationWorkItem {
                instance_id: instance_id.into(),
                ..Default::default()
            };
            let state = be.get_orchestration_runtime_state(&owi).await?;
            if state.new_events().is_empty() && state.old_events.is_empty() {
                return Err(BackendError::TaskHubNotFound);
            }
            if !state.is_completed() {
                return Err(BackendError::NotInitialized);
            }
            let sub_orchestration_instances =
                get_sub_orchestration_instances(&state.old_events, state.new_events());
            for sub_instance_id in sub_orchestration_instances {
                let sub_result =
                    purge_orchestration_state(be, &InstanceID(sub_instance_id), recursive).await?;
                deleted_instance_count += sub_result;
            }
        }
        be.purge_orchestration_state(instance_id).await?;
        Ok(deleted_instance_count + 1)
    })
}

#[allow(dead_code)] // TODO: Remove
pub(crate) async fn terminate_sub_orchestration_instances(
    be: &dyn Backend,
    /* Unused
    instance_id: InstanceID,
     */
    state: &OrchestrationRuntimeState,
    et: &ExecutionTerminatedEvent,
) -> Result<(), Box<dyn Error>> {
    if !et.recurse {
        return Ok(());
    }
    let sub_orchestration_instances =
        get_sub_orchestration_instances(&state.old_events, state.new_events());
    for sub_instance_id in sub_orchestration_instances {
        let e = new_execution_terminated_event(et.input.as_deref(), et.recurse);
        be.add_new_orchestration_event(&sub_instance_id, &e).await?;
    }
    Ok(())
}

fn get_sub_orchestration_instances(
    old_events: &[HistoryEvent],
    new_events: &[HistoryEvent],
) -> Vec<String> {
    let mut sub_orchestration_instances_map = HashMap::new();

    for e in old_events.iter().chain(new_events.iter()) {
        if let Some(instance_id) = get_sub_orchestration_instance_id(e) {
            sub_orchestration_instances_map.insert(instance_id, ());
        }
    }

    sub_orchestration_instances_map.into_keys().collect()
}

fn get_sub_orchestration_instance_id(event: &HistoryEvent) -> Option<String> {
    match &event.event_type {
        Some(SubOrchestrationInstanceCreated(created)) => Some(created.instance_id.clone()),
        _ => None,
    }
}
