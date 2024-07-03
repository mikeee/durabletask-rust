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
use prost_wkt_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::durabletask_pb::{OrchestrationStatus, TaskFailureDetails};

pub static ERR_INSTANCE_NOT_FOUND: &str = "no such instance exists";
pub static ERR_NOT_STARTED: &str = "orchestration has not started";
pub static ERR_NOT_COMPLETED: &str = "orchestration has not yet completed";
pub static ERR_NO_FAILURES: &str = "orchestration did not report failure details";
pub static ERR_DUPLICATE_INSTANCE: &str = "orchestration instance already exists";
pub static ERR_IGNORE_INSTANCE: &str = "ignore creating orchestration instance";

pub type NewOrchestration = crate::durabletask_pb::CreateInstanceRequest;

impl NewOrchestration {
    pub fn builder() -> NewOrchestrationBuilder {
        NewOrchestrationBuilder::default()
    }
}

pub type OrchestrationIdReusePolicy = crate::durabletask_pb::OrchestrationIdReusePolicy;

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct InstanceID(pub String);

#[derive(Default, Debug, PartialEq)]
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
        let bytes = match serde_json::to_vec(input) {
            Ok(b) => b,
            Err(_) => vec![],
        };

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
        }
    }
}

pub type FetchOrchestrationMetadata = crate::durabletask_pb::GetInstanceRequest;

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

pub type RaiseEvent = crate::durabletask_pb::RaiseEventRequest;

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
        let bytes = match serde_json::to_vec(payload) {
            Ok(b) => b,
            Err(_) => vec![],
        };

        self.input = Some(String::from_utf8(bytes).unwrap());
        self
    }

    pub fn raw_event_data(mut self, payload: String) -> Self {
        self.input = Some(payload);
        self
    }
}

pub type Terminate = crate::durabletask_pb::TerminateRequest;

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
        let bytes = match serde_json::to_vec(data) {
            Ok(b) => b,
            Err(_) => vec![],
        };

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

pub type Purge = crate::durabletask_pb::PurgeInstancesRequest;

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

#[derive(Default, Debug, Serialize, Deserialize)]
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

// TODO: implement builder pattern?
#[allow(clippy::too_many_arguments)]
impl OrchestrationMetadata {
    pub fn new(
        instance_id: InstanceID,
        name: String,
        status: OrchestrationStatus,
        created_at: Timestamp,
        last_updated_at: Timestamp,
        serialized_input: Option<String>,
        serialized_output: Option<String>,
        serialized_custom_status: Option<String>,
        failure_details: Option<TaskFailureDetails>,
    ) -> Self {
        OrchestrationMetadata {
            instance_id,
            name,
            runtime_status: status,
            created_at,
            last_updated_at,
            serialized_input,
            serialized_output,
            serialized_custom_status,
            failure_details,
        }
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

    use crate::durabletask_pb::{OrchestrationStatus, TaskFailureDetails};

    use super::*;

    #[test]
    fn test_new_orchestration_builder() {
        let instance_id = InstanceID("test-instance".to_string());
        let input = serde_json::json!({"key": "value"});
        let time = Timestamp::default();

        let new_orchestration = NewOrchestration::builder()
            .instance_id(instance_id.clone())
            .input(&input)
            .start_time(time.clone())
            .build();

        assert_eq!(new_orchestration.instance_id, instance_id.0);
        assert_eq!(
            new_orchestration.input,
            Some(r#"{"key":"value"}"#.to_string())
        );
        assert_eq!(new_orchestration.scheduled_start_timestamp, Some(time));
    }

    #[test]
    fn test_new_orchestration_builder_raw_input() {
        let instance_id = InstanceID("test-instance".to_string());
        let raw_input = r#"{"key":"value"}"#.to_string();

        let new_orchestration = NewOrchestration::builder()
            .instance_id(instance_id)
            .raw_input(raw_input.clone())
            .build();

        assert_eq!(new_orchestration.input, Some(raw_input));
    }

    #[test]
    fn test_new_orchestration_builder_reuse_policy() {
        let reuse_policy = OrchestrationIdReusePolicy {
            action: 0,
            operation_status: vec![0],
        };

        let new_orchestration = NewOrchestration::builder()
            .orchestration_id_reuse_policy(reuse_policy.clone())
            .build();

        assert_eq!(
            new_orchestration.orchestration_id_reuse_policy,
            Some(reuse_policy)
        );
    }

    #[test]
    fn test_fetch_orchestration_metadata_builder() {
        let builder = FetchOrchestrationMetadataBuilder::new().fetch_payloads(true);
        assert_eq!(builder.get_inputs_and_outputs, Some(true));
    }

    #[test]
    fn test_raise_event_builder() {
        let event_data = serde_json::json!({"event": "data"});
        let builder = RaiseEventBuilder::new().event_payload(&event_data);
        assert_eq!(builder.input, Some(r#"{"event":"data"}"#.to_string()));
    }

    #[test]
    fn test_raise_event_builder_raw_data() {
        let raw_data = r#"{"event":"data"}"#.to_string();
        let builder = RaiseEventBuilder::new().raw_event_data(raw_data.clone());
        assert_eq!(builder.input, Some(raw_data));
    }

    #[test]
    fn test_terminate_builder() {
        let output_data = serde_json::json!({"output": "data"});
        let builder = TerminateBuilder::new()
            .output(&output_data)
            .recursive_terminate(true);
        assert_eq!(builder.output, Some(r#"{"output":"data"}"#.to_string()));
        assert_eq!(builder.recursive, Some(true));
    }

    #[test]
    fn test_terminate_builder_raw_output() {
        let raw_output = r#"{"output":"data"}"#.to_string();
        let builder = TerminateBuilder::new()
            .raw_output(raw_output.clone())
            .recursive_terminate(true);
        assert_eq!(builder.output, Some(raw_output));
        assert_eq!(builder.recursive, Some(true));
    }

    #[test]
    fn test_purge_builder() {
        let builder = PurgeBuilder::new().recursive_purge(true);
        assert_eq!(builder.recursive, Some(true));
    }

    #[test]
    fn test_orchestration_metadata_is_running() {
        let metadata = OrchestrationMetadata {
            instance_id: InstanceID("test".to_string()),
            name: "test".to_string(),
            runtime_status: OrchestrationStatus::Running,
            created_at: Timestamp::default(),
            last_updated_at: Timestamp::default(),
            serialized_input: None,
            serialized_output: None,
            serialized_custom_status: None,
            failure_details: None,
        };

        assert!(metadata.is_running());
        assert!(!metadata.is_complete());
    }

    #[test]
    fn test_orchestration_metadata_is_complete() {
        let metadata = OrchestrationMetadata {
            instance_id: InstanceID("test".to_string()),
            name: "test".to_string(),
            runtime_status: OrchestrationStatus::Completed,
            created_at: Timestamp::default(),
            last_updated_at: Timestamp::default(),
            serialized_input: None,
            serialized_output: None,
            serialized_custom_status: None,
            failure_details: None,
        };

        assert!(!metadata.is_running());
        assert!(metadata.is_complete());
    }

    #[test]
    fn test_orchestration_metadata_new() {
        let instance_id = InstanceID("test".to_string());
        let name = "test".to_string();
        let status = OrchestrationStatus::Running;
        let created_at = Timestamp::default();
        let last_updated_at = Timestamp::default();
        let serialized_input = Some("input".to_string());
        let serialized_output = Some("output".to_string());
        let serialized_custom_status = Some("custom".to_string());
        let failure_details = Some(TaskFailureDetails::default());

        let metadata = OrchestrationMetadata::new(
            instance_id.clone(),
            name.clone(),
            status,
            created_at.clone(),
            last_updated_at.clone(),
            serialized_input.clone(),
            serialized_output.clone(),
            serialized_custom_status.clone(),
            failure_details.clone(),
        );

        assert_eq!(metadata.instance_id, instance_id);
        assert_eq!(metadata.name, name);
        assert_eq!(metadata.runtime_status, status);
        assert_eq!(metadata.created_at, created_at);
        assert_eq!(metadata.last_updated_at, last_updated_at);
        assert_eq!(metadata.serialized_input, serialized_input);
        assert_eq!(metadata.serialized_output, serialized_output);
        assert_eq!(metadata.serialized_custom_status, serialized_custom_status);
        assert_eq!(metadata.failure_details, failure_details);
    }
}
