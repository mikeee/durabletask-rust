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
use std::time::Duration;

use crate::api::InstanceID;
use crate::backend::runtimestate::OrchestrationRuntimeState;
use crate::durabletask_pb::history_event::EventType;
use crate::durabletask_pb::HistoryEvent;

#[allow(dead_code)] // TODO: Remove
#[derive(Debug)]
struct NoWorkItemsError;

impl fmt::Display for NoWorkItemsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no work items were found")
    }
}

impl Error for NoWorkItemsError {}

#[allow(dead_code)] // TODO: Remove
trait WorkItem: fmt::Display {
    fn is_work_item(&self) -> bool {
        true
    }
}

#[allow(dead_code)] // TODO: Remove
#[derive(Default)]
pub(crate) struct OrchestrationWorkItem {
    pub instance_id: InstanceID,
    pub new_events: Vec<HistoryEvent>,
    pub locked_by: String,
    pub retry_count: i32,
    pub state: OrchestrationRuntimeState,
    pub properties: HashMap<String, Box<dyn std::any::Any>>,
}

impl fmt::Display for OrchestrationWorkItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.instance_id, self.new_events.len())
    }
}

impl WorkItem for OrchestrationWorkItem {}

#[allow(dead_code)] // TODO: Remove
impl OrchestrationWorkItem {
    fn abandon_delay(&self) -> Duration {
        match self.retry_count {
            0 => Duration::from_secs(0),
            retry_count if retry_count > 100 => Duration::from_secs(5 * 60),
            retry_count => Duration::from_secs(retry_count as u64),
        }
    }
}

#[allow(dead_code)] // TODO: Remove
pub(crate) struct ActivityWorkItem {
    pub sequence_number: i64,
    pub instance_id: InstanceID,
    pub new_event: HistoryEvent,
    pub result: Option<HistoryEvent>,
    pub locked_by: String,
    pub properties: HashMap<String, Box<dyn std::any::Any>>,
}

impl fmt::Display for ActivityWorkItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match &self.new_event.event_type {
            Some(EventType::TaskScheduled(scheduled_task)) => &scheduled_task.name,
            _ => todo!("handle other event cases"),
        };
        let task_id = self.new_event.event_id;
        write!(f, "{}/{:#?}#{}", self.instance_id, name, task_id)
    }
}

impl WorkItem for ActivityWorkItem {}
