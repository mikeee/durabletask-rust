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
use std::fmt;

use crate::durabletask_pb::{history_event::EventType, HistoryEvent};

use super::runtimestate::OrchestrationRuntimeState;

pub trait WorkItem {
    fn is_work_item(&self) -> bool;
}

#[allow(dead_code)] // TODO: revisit dead fields
#[derive(Default)]
pub struct OrchestrationWorkItem {
    pub instance_id: String,
    new_events: Vec<HistoryEvent>,
    locked_by: String,
    retry_count: i32,
    state: OrchestrationRuntimeState,
    properties: std::collections::HashMap<String, Box<dyn std::any::Any>>,
}

impl fmt::Display for OrchestrationWorkItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} event(s))",
            self.instance_id,
            self.new_events.len()
        )
    }
}

impl WorkItem for OrchestrationWorkItem {
    fn is_work_item(&self) -> bool {
        true
    }
}

impl OrchestrationWorkItem {
    pub fn get_abandon_delay(&self) -> std::time::Duration {
        match self.retry_count {
            0 => std::time::Duration::from_secs(0), // no delay
            n if n > 100 => std::time::Duration::from_secs(300), // max delay
            n => std::time::Duration::from_secs(n as u64), // linear backoff
        }
    }
}

#[allow(dead_code)] // TODO: Revisit dead fields
pub struct ActivityWorkItem {
    sequence_number: i64,
    instance_id: String,
    new_event: HistoryEvent,
    result: Option<HistoryEvent>,
    locked_by: String,
    properties: std::collections::HashMap<String, Box<dyn std::any::Any>>,
}

impl fmt::Display for ActivityWorkItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self.new_event.event_type {
            Some(EventType::TaskScheduled(scheduled_task)) => &scheduled_task.name,
            _ => todo!("handle other event cases"),
        };
        let task_id = self.new_event.event_id;
        write!(f, "{}/{:#?}#{}", self.instance_id, name, task_id)
    }
}

impl WorkItem for ActivityWorkItem {
    fn is_work_item(&self) -> bool {
        true
    }
}
