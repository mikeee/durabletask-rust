use std::fmt;

use crate::durabletask_pb::{history_event::EventType, HistoryEvent};

use super::runtimestate::OrchestrationRuntimeState;

#[allow(dead_code)] // TODO: revisit dead fields
pub struct OrchestrationWorkItem {
    instance_id: String,
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

impl OrchestrationWorkItem {
    pub fn is_work_item(&self) -> bool {
        true
    }

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

impl ActivityWorkItem {
    pub fn is_work_item(&self) -> bool {
        true
    }
}
