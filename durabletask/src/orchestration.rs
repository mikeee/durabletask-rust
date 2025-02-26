use crate::client::OrchestrationStatus;
use durabletask_proto::TaskFailureDetails;
use std::time::{Duration, SystemTime};

pub struct Orchestration {
    pub instance_id: String,
    pub name: String,
    pub runtime_status: OrchestrationStatus,
    pub created_at: Duration,
    pub last_updated_at: Duration,
    pub serialised_input: Option<String>,
    pub serialized_output: Option<String>,
    pub serialized_custom_status: Option<String>,
    pub failure_details: Option<TaskFailureDetails>,
}

#[allow(clippy::too_many_arguments)]
impl Orchestration {
    pub fn new(
        instance_id: String,
        name: String,
        runtime_status: OrchestrationStatus,
        created_at: Duration,
        last_updated_at: Duration,
        serialised_input: Option<String>,
        serialized_output: Option<String>,
        serialized_custom_status: Option<String>,
        failure_details: Option<TaskFailureDetails>,
    ) -> Self {
        Self {
            instance_id,
            name,
            runtime_status,
            created_at,
            last_updated_at,
            serialised_input,
            serialized_output,
            serialized_custom_status,
            failure_details,
        }
    }

    pub fn raised_if_failed(&mut self) {
        if self.failure_details.is_some() {
            panic!("Orchestration failed: {:?}", self.failure_details);
        }
    }
}

pub struct PurgeResult {
    pub deleted_instance_count: usize,
}

impl PurgeResult {
    pub fn new(deleted_instance_count: usize) -> Self {
        Self {
            deleted_instance_count,
        }
    }
}

pub struct PurgeInstanceCriteria {
    created_time_from: Option<SystemTime>,
    created_time_to: Option<SystemTime>,
    runtime_status_list: Vec<OrchestrationStatus>,
    timeout: usize, // default 5 minutes
}

impl Default for PurgeInstanceCriteria {
    fn default() -> Self {
        Self::new()
    }
}

impl PurgeInstanceCriteria {
    pub fn new() -> Self {
        Self {
            created_time_from: None,
            created_time_to: None,
            runtime_status_list: Vec::new(),
            timeout: 5 * 60 * 10000,
        }
    }

    pub fn set_created_time_to(&mut self, date: SystemTime) {
        self.created_time_to = Some(date);
    }

    pub fn get_created_time_to(&self) -> Option<SystemTime> {
        self.created_time_to
    }

    pub fn set_created_time_from(&mut self, date: SystemTime) {
        self.created_time_from = Some(date);
    }

    pub fn get_created_time_from(&self) -> Option<SystemTime> {
        self.created_time_from
    }

    pub fn set_runtime_status_list(&mut self, status_list: Vec<OrchestrationStatus>) {
        self.runtime_status_list = status_list;
    }

    pub fn get_runtime_status_list(&self) -> Vec<OrchestrationStatus> {
        self.runtime_status_list.clone()
    }

    pub fn set_timeout(&mut self, timeout: usize) {
        self.timeout = timeout;
    }

    pub fn get_timeout(&self) -> usize {
        self.timeout
    }
}
