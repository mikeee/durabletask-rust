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

use prost_wkt_types::Timestamp;
use serde::{Deserialize, Serialize};

use durabletask_proto::{CreateOrchestrationAction, OrchestrationStatus, TaskFailureDetails};

pub static ERR_INSTANCE_NOT_FOUND: &str = "no such instance exists";
pub static ERR_NOT_STARTED: &str = "orchestration has not started";
pub static ERR_NOT_COMPLETED: &str = "orchestration has not yet completed";
pub static ERR_NO_FAILURES: &str = "orchestration did not report failure details";
pub static ERR_DUPLICATE_INSTANCE: &str = "orchestration instance already exists";
pub static ERR_IGNORE_INSTANCE: &str = "ignore creating orchestration instance";

pub static REUSE_ID_ACTION_ERROR: CreateOrchestrationAction = CreateOrchestrationAction::Error;
pub static REUSE_ID_ACTION_IGNORE: CreateOrchestrationAction = CreateOrchestrationAction::Ignore;
pub static REUSE_ID_ACTION_TERMINATE: CreateOrchestrationAction =
    CreateOrchestrationAction::Terminate;

pub static RUNTIME_STATUS_RUNNING: OrchestrationStatus = OrchestrationStatus::Running;
pub static RUNTIME_STATUS_COMPLETED: OrchestrationStatus = OrchestrationStatus::Completed;
pub static RUNTIME_STATUS_CONTINUED_AS_NEW: OrchestrationStatus =
    OrchestrationStatus::ContinuedAsNew;
pub static RUNTIME_STATUS_FAILED: OrchestrationStatus = OrchestrationStatus::Failed;
pub static RUNTIME_STATUS_CANCELED: OrchestrationStatus = OrchestrationStatus::Canceled;
pub static RUNTIME_STATUS_TERMINATED: OrchestrationStatus = OrchestrationStatus::Terminated;
pub static RUNTIME_STATUS_PENDING: OrchestrationStatus = OrchestrationStatus::Pending;
pub static RUNTIME_STATUS_SUSPENDED: OrchestrationStatus = OrchestrationStatus::Suspended;

pub type NewOrchestration = durabletask_proto::CreateInstanceRequest;

pub type OrchestrationIdReusePolicy = durabletask_proto::OrchestrationIdReusePolicy;

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct InstanceID(pub String);

impl fmt::Display for InstanceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&InstanceID> for InstanceID {
    fn from(value: &InstanceID) -> Self {
        InstanceID(value.to_string())
    }
}

#[derive(Default, PartialEq)]
pub struct NewOrchestrationBuilder {
    instance_id: Option<InstanceID>,
    orchestration_id_reuse_policy: Option<OrchestrationIdReusePolicy>,
    input: Option<String>,
    scheduled_start_timestamp: Option<Timestamp>,
}

impl NewOrchestrationBuilder {
    pub fn new() -> Self {
        NewOrchestrationBuilder {
            ..Default::default()
        }
    }

    pub fn instance_id(mut self, id: InstanceID) -> Self {
        self.instance_id = Some(id);
        self
    }

    pub fn orchestration_id_reuse_policy(mut self, policy: OrchestrationIdReusePolicy) -> Self {
        self.orchestration_id_reuse_policy = Some(policy);
        self
    }

    pub fn input<T: Serialize>(mut self, input: &T) -> Self {
        let bytes = serde_json::to_vec(input).unwrap_or_else(|_| vec![]);

        self.input = Some(String::from_utf8(bytes).unwrap());
        self
    }

    pub fn raw_input(mut self, input: String) -> Self {
        self.input = Some(input);
        self
    }

    pub fn start_time(mut self, time: Timestamp) -> Self {
        self.scheduled_start_timestamp = Some(time);
        self
    }

    pub fn build(self) -> NewOrchestration {
        let instance_id = match self.instance_id {
            None => "".to_string(),
            Some(id) => id.0,
        };

        NewOrchestration {
            instance_id,
            name: "name".to_string(),
            version: None,
            input: self.input,
            scheduled_start_timestamp: self.scheduled_start_timestamp,
            orchestration_id_reuse_policy: self.orchestration_id_reuse_policy,
            execution_id: None,       // TODO: implement execution id
            tags: Default::default(), // TODO: implement tags
        }
    }
}

pub type FetchOrchestrationMetadata = durabletask_proto::GetInstanceRequest;

#[derive(Default, Debug, PartialEq)]
pub struct FetchOrchestrationMetadataBuilder {
    get_inputs_and_outputs: Option<bool>,
}

impl FetchOrchestrationMetadataBuilder {
    pub fn new() -> Self {
        FetchOrchestrationMetadataBuilder {
            ..Default::default()
        }
    }

    pub fn fetch_payloads(mut self, fetch_payloads: bool) -> Self {
        self.get_inputs_and_outputs = Some(fetch_payloads);
        self
    }
}

pub type RaiseEvent = durabletask_proto::RaiseEventRequest;

#[derive(Default, Debug, PartialEq)]
pub struct RaiseEventBuilder {
    input: Option<String>,
}

impl RaiseEventBuilder {
    pub fn new() -> Self {
        RaiseEventBuilder {
            ..Default::default()
        }
    }

    pub fn event_payload<T: Serialize>(mut self, payload: &T) -> Self {
        let bytes = serde_json::to_vec(payload).unwrap_or_else(|_| vec![]);

        self.input = Some(String::from_utf8(bytes).unwrap());
        self
    }

    pub fn raw_event_data(mut self, payload: String) -> Self {
        self.input = Some(payload);
        self
    }
}

pub type Terminate = durabletask_proto::TerminateRequest;

#[derive(Default, Debug, PartialEq)]
pub struct TerminateBuilder {
    output: Option<String>,
    recursive: Option<bool>,
}

impl TerminateBuilder {
    pub fn new() -> Self {
        TerminateBuilder {
            ..Default::default()
        }
    }

    pub fn output<T: Serialize>(mut self, data: &T) -> Self {
        let bytes = serde_json::to_vec(data).unwrap_or_else(|_| vec![]);

        self.output = Some(String::from_utf8(bytes).unwrap());
        self
    }

    pub fn raw_output(mut self, data: String) -> Self {
        self.output = Some(data);
        self
    }

    pub fn recursive_terminate(mut self, recursive: bool) -> Self {
        self.recursive = Some(recursive);
        self
    }
}

pub type Purge = durabletask_proto::PurgeInstancesRequest;

#[derive(Default, Debug, PartialEq)]
pub struct PurgeBuilder {
    recursive: Option<bool>,
}

impl PurgeBuilder {
    pub fn new() -> Self {
        PurgeBuilder {
            ..Default::default()
        }
    }

    pub fn recursive_purge(mut self, recursive: bool) -> Self {
        self.recursive = Some(recursive);
        self
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct OrchestrationMetadata {
    #[serde(rename = "id")]
    pub instance_id: InstanceID,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "status")]
    pub runtime_status: OrchestrationStatus,
    #[serde(rename = "createdAt")]
    pub created_at: Timestamp,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: Timestamp,
    #[serde(rename = "serializedInput")]
    pub serialized_input: Option<String>,
    #[serde(rename = "serializedOutput")]
    pub serialized_output: Option<String>,
    #[serde(rename = "serializedCustomStatus")]
    pub serialized_custom_status: Option<String>,
    #[serde(rename = "failureDetails")]
    pub failure_details: Option<TaskFailureDetails>,
}

pub struct OrchestrationMetadataBuilder {
    instance_id: Option<InstanceID>,
    name: Option<String>,
    status: Option<OrchestrationStatus>,
    created_at: Option<Timestamp>,
    last_updated_at: Option<Timestamp>,
    serialized_input: Option<String>,
    serialized_output: Option<String>,
    serialized_custom_status: Option<String>,
    failure_details: Option<TaskFailureDetails>,
}

impl Default for OrchestrationMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestrationMetadataBuilder {
    pub fn new() -> Self {
        OrchestrationMetadataBuilder {
            instance_id: None,
            name: None,
            status: None,
            created_at: None,
            last_updated_at: None,
            serialized_input: None,
            serialized_output: None,
            serialized_custom_status: None,
            failure_details: None,
        }
    }

    pub fn instance_id(mut self, instance_id: InstanceID) -> Self {
        self.instance_id = Some(instance_id);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn status(mut self, status: OrchestrationStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn created_at(mut self, created_at: Timestamp) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn last_updated_at(mut self, last_updated_at: Timestamp) -> Self {
        self.last_updated_at = Some(last_updated_at);
        self
    }

    pub fn serialized_input(mut self, serialized_input: String) -> Self {
        self.serialized_input = Some(serialized_input);
        self
    }

    pub fn serialized_output(mut self, serialized_output: String) -> Self {
        self.serialized_output = Some(serialized_output);
        self
    }

    pub fn serialized_custom_status(mut self, serialized_custom_status: String) -> Self {
        self.serialized_custom_status = Some(serialized_custom_status);
        self
    }

    pub fn failure_details(mut self, failure_details: TaskFailureDetails) -> Self {
        self.failure_details = Some(failure_details);
        self
    }

    pub fn build(self) -> Result<OrchestrationMetadata, &'static str> {
        let instance_id = self.instance_id.ok_or("instance_id is required")?;
        let name = self.name.ok_or("name is required")?;
        let status = self.status.ok_or("status is required")?;
        let created_at = self.created_at.ok_or("created_at is required")?;
        let last_updated_at = self.last_updated_at.ok_or("last_updated_at is required")?;

        Ok(OrchestrationMetadata {
            instance_id,
            name,
            runtime_status: status,
            created_at,
            last_updated_at,
            serialized_input: self.serialized_input,
            serialized_output: self.serialized_output,
            serialized_custom_status: self.serialized_custom_status,
            failure_details: self.failure_details,
        })
    }
}

impl OrchestrationMetadata {
    pub fn builder() -> OrchestrationMetadataBuilder {
        OrchestrationMetadataBuilder::new()
    }

    pub fn is_running(&self) -> bool {
        !self.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        matches!(
            self.runtime_status,
            OrchestrationStatus::Completed
                | OrchestrationStatus::Failed
                | OrchestrationStatus::Terminated
                | OrchestrationStatus::Canceled
        )
    }
}

#[cfg(test)]
mod tests {
    use prost_wkt_types::Timestamp;

    use super::*;

    #[test]
    fn test_new_orchestration_builder() {
        let instance_id = InstanceID("test-id".to_string());
        let input = "test input";
        let time = Timestamp::date_time(2024, 1, 1, 0, 0, 0).unwrap();

        let new_orchestration = NewOrchestrationBuilder::new()
            .instance_id(instance_id.clone())
            .orchestration_id_reuse_policy(OrchestrationIdReusePolicy::default())
            .raw_input(input.to_string())
            .start_time(time.clone())
            .build();

        assert_eq!(new_orchestration.instance_id, instance_id.0);
        assert_eq!(new_orchestration.input, Some(input.to_string()));
        assert_eq!(new_orchestration.scheduled_start_timestamp, Some(time));
        assert_eq!(
            new_orchestration.orchestration_id_reuse_policy,
            Some(OrchestrationIdReusePolicy::default())
        );
    }

    #[test]
    fn test_fetch_orchestration_metadata_builder() {
        let builder = FetchOrchestrationMetadataBuilder::new().fetch_payloads(true);

        assert_eq!(builder.get_inputs_and_outputs, Some(true));
    }

    #[test]
    fn test_raise_event_builder() {
        let payload = "test event payload";
        let builder = RaiseEventBuilder::new().raw_event_data(payload.to_string());

        assert_eq!(builder.input, Some(payload.to_string()));
    }

    #[test]
    fn test_terminate_builder() {
        let output = "test output";
        let builder = TerminateBuilder::new()
            .raw_output(output.to_string())
            .recursive_terminate(true);

        assert_eq!(builder.output, Some(output.to_string()));
        assert_eq!(builder.recursive, Some(true));
    }

    #[test]
    fn test_purge_builder() {
        let builder = PurgeBuilder::new().recursive_purge(true);

        assert_eq!(builder.recursive, Some(true));
    }

    #[test]
    fn test_orchestration_metadata_builder() {
        let instance_id = InstanceID("test-id".to_string());
        let name = "test-name".to_string();
        let status = OrchestrationStatus::Running;
        let created_at = Timestamp::date_time(2024, 1, 1, 0, 0, 0).unwrap();
        let last_updated_at = Timestamp::date_time(2024, 1, 2, 0, 0, 0).unwrap();

        let metadata = OrchestrationMetadata::builder()
            .instance_id(instance_id.clone())
            .name(name.clone())
            .status(status)
            .created_at(created_at.clone())
            .last_updated_at(last_updated_at.clone())
            .serialized_input("test input".to_string())
            .build()
            .unwrap();

        assert_eq!(metadata.instance_id, instance_id);
        assert_eq!(metadata.name, name);
        assert_eq!(metadata.runtime_status, status);
        assert_eq!(metadata.created_at, created_at);
        assert_eq!(metadata.last_updated_at, last_updated_at);
        assert_eq!(metadata.serialized_input, Some("test input".to_string()));
    }

    #[test]
    fn test_orchestration_metadata_is_running() {
        let mut metadata = OrchestrationMetadata::default();
        metadata.runtime_status = OrchestrationStatus::Running;
        assert!(metadata.is_running());
        assert!(!metadata.is_complete());
    }

    #[test]
    fn test_orchestration_metadata_is_complete() {
        let statuses = vec![
            OrchestrationStatus::Completed,
            OrchestrationStatus::Failed,
            OrchestrationStatus::Terminated,
            OrchestrationStatus::Canceled,
        ];

        for status in statuses {
            let mut metadata = OrchestrationMetadata::default();
            metadata.runtime_status = status;
            assert!(!metadata.is_running());
            assert!(metadata.is_complete());
        }
    }
}
