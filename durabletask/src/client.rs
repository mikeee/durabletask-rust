use crate::api::{
    FetchOrchestrationMetadata, NewOrchestration, OrchestrationMetadata,
    OrchestrationMetadataBuilder, Purge, RaiseEvent, Terminate,
};
use durabletask_proto::{
    CreateInstanceRequest, CreateInstanceResponse, GetInstanceRequest, GetInstanceResponse,
    PurgeInstancesRequest, PurgeInstancesResponse, RaiseEventRequest, RaiseEventResponse,
    ResumeRequest, ResumeResponse, SuspendRequest, SuspendResponse, TerminateRequest,
    TerminateResponse,
};
use std::time::Duration;
use tonic::transport::Channel;

// TODO: Remove placeholder
#[derive(Debug, Clone)]
pub struct TempTaskFailureDetails {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct OrchestrationFailedError {
    pub message: String,
    pub details: Option<TempTaskFailureDetails>,
}

impl OrchestrationFailedError {
    pub fn new(message: String, details: Option<TempTaskFailureDetails>) -> Self {
        Self { message, details }
    }
}

pub enum Error {
    OrchestrationError(OrchestrationFailedError),
    TransportError(tonic::transport::Error),
    StatusError(tonic::Status),
}

impl From<OrchestrationFailedError> for Error {
    fn from(e: OrchestrationFailedError) -> Self {
        Error::OrchestrationError(e)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(e: tonic::transport::Error) -> Self {
        Error::TransportError(e)
    }
}

impl From<tonic::Status> for Error {
    fn from(e: tonic::Status) -> Self {
        Error::StatusError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum OrchestrationStatus {
    Running,
    Completed,
    Failed,
    Terminated,
    ContinuedAsNew,
    Pending,
    Suspended,
}

impl From<durabletask_proto::OrchestrationStatus> for OrchestrationStatus {
    fn from(status: durabletask_proto::OrchestrationStatus) -> Self {
        match status {
            durabletask_proto::OrchestrationStatus::Running => OrchestrationStatus::Running,
            durabletask_proto::OrchestrationStatus::Completed => OrchestrationStatus::Completed,
            durabletask_proto::OrchestrationStatus::Failed => OrchestrationStatus::Failed,
            durabletask_proto::OrchestrationStatus::Terminated => OrchestrationStatus::Terminated,
            durabletask_proto::OrchestrationStatus::ContinuedAsNew => {
                OrchestrationStatus::ContinuedAsNew
            }
            durabletask_proto::OrchestrationStatus::Pending => OrchestrationStatus::Pending,
            durabletask_proto::OrchestrationStatus::Suspended => OrchestrationStatus::Suspended,
            _ => panic!("Unknown OrchestrationStatus"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrchestrationState {
    pub instance_id: String,
    pub name: String,
    pub runtime_status: OrchestrationStatus,
    pub created_at: Duration,
    pub last_updated_at: Duration,
    pub serialized_input: Option<String>,
    pub serialized_output: Option<String>,
    pub serialized_custom_status: Option<String>,
    pub failure_details: Option<TempTaskFailureDetails>,
}

impl OrchestrationState {
    pub fn raise_if_failed(&mut self) -> Result<()> {
        if let Some(details) = self.clone().failure_details {
            return Err(Error::OrchestrationError(OrchestrationFailedError {
                message: format!(
                    "Orchestration '{}' failed: {}",
                    self.instance_id, details.message
                ),
                details: Option::from(details),
            }));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TaskHubGrpcClient {
    client:
        durabletask_proto::task_hub_sidecar_service_client::TaskHubSidecarServiceClient<Channel>,
}

async fn new_client(
    address: String,
) -> Result<durabletask_proto::task_hub_sidecar_service_client::TaskHubSidecarServiceClient<Channel>>
{
    Ok(
        durabletask_proto::task_hub_sidecar_service_client::TaskHubSidecarServiceClient::connect(
            address,
        )
        .await?,
    )
}

impl TaskHubGrpcClient {
    pub async fn connect(address: String) -> Result<Self> {
        let client = new_client(address).await?;
        Ok(Self { client })
    }
    pub async fn start_instance(
        &mut self,
        orchestrator: &str,
        options: Option<NewOrchestration>,
    ) -> Result<CreateInstanceResponse> {
        let options: NewOrchestration = options.expect("failed to parse options");
        let resp = self
            .client
            .start_instance(CreateInstanceRequest {
                instance_id: options.instance_id,
                name: orchestrator.parse().unwrap(),
                version: options.version,
                input: options.input,
                scheduled_start_timestamp: options.scheduled_start_timestamp,
                orchestration_id_reuse_policy: options.orchestration_id_reuse_policy,
                execution_id: options.execution_id,
                tags: Default::default(),
            })
            .await?
            .into_inner();
        Ok(resp)
    }

    pub async fn get_instance(
        &mut self,
        instance_id: crate::api::InstanceID,
        options: Option<FetchOrchestrationMetadata>,
    ) -> Result<GetInstanceResponse> {
        let options: FetchOrchestrationMetadata = options.expect("failed to parse options");
        let resp = self
            .client
            .get_instance(GetInstanceRequest {
                instance_id: instance_id.to_string(),
                get_inputs_and_outputs: options.get_inputs_and_outputs,
            })
            .await?
            .into_inner();
        Ok(resp)
    }

    pub async fn wait_for_instance_start(
        &mut self,
        instance_id: crate::api::InstanceID,
        options: Option<FetchOrchestrationMetadata>,
    ) -> Result<OrchestrationMetadata> {
        let options: FetchOrchestrationMetadata = options.expect("failed to parse options");
        let fetch_payloads = options.get_inputs_and_outputs;

        let request = GetInstanceRequest {
            instance_id: instance_id.to_string(),
            get_inputs_and_outputs: fetch_payloads,
        };

        // TODO: Add retries

        match self.client.wait_for_instance_start(request).await {
            Ok(response) => Ok(OrchestrationMetadataBuilder::from(response.into_inner())
                .build()
                .unwrap()),
            Err(status) => {
                if status.code() != tonic::Code::Ok {
                    Err(Error::OrchestrationError(OrchestrationFailedError {
                        message: format!(
                            "Orchestration '{}' failed - code: {}",
                            instance_id,
                            status.code()
                        ),
                        details: None,
                    }))
                } else {
                    Err(Error::StatusError(status))
                }
            }
        }
    }

    pub async fn wait_for_instance_completion(
        &mut self,
        instance_id: crate::api::InstanceID,
        options: Option<FetchOrchestrationMetadata>,
    ) -> Result<OrchestrationMetadata> {
        let options: FetchOrchestrationMetadata = options.expect("failed to parse options");
        let fetch_payloads = options.get_inputs_and_outputs;

        let request = GetInstanceRequest {
            instance_id: instance_id.to_string(),
            get_inputs_and_outputs: fetch_payloads,
        };

        // TODO: Add retries

        match self.client.wait_for_instance_completion(request).await {
            Ok(response) => Ok(OrchestrationMetadataBuilder::from(response.into_inner())
                .build()
                .unwrap()),
            Err(status) => {
                if status.code() != tonic::Code::Ok {
                    Err(Error::OrchestrationError(OrchestrationFailedError {
                        message: format!(
                            "Orchestration '{}' failed - code: {}",
                            instance_id,
                            status.code()
                        ),
                        details: None,
                    }))
                } else {
                    Err(Error::StatusError(status))
                }
            }
        }
    }

    pub async fn raise_event(
        &mut self,
        instance_id: crate::api::InstanceID,
        event_name: String,
        options: Option<RaiseEvent>,
    ) -> Result<RaiseEventResponse> {
        let options: RaiseEvent = options.expect("failed to parse options");
        let resp = self
            .client
            .raise_event(RaiseEventRequest {
                instance_id: instance_id.to_string(),
                name: event_name,
                input: options.input,
            })
            .await?
            .into_inner();
        Ok(resp)
    }

    pub async fn terminate_instance(
        &mut self,
        instance_id: crate::api::InstanceID,
        options: Option<Terminate>,
    ) -> Result<TerminateResponse> {
        let options: Terminate = options.expect("failed to parse options");
        let resp = self
            .client
            .terminate_instance(TerminateRequest {
                instance_id: instance_id.to_string(),
                output: options.output,
                recursive: options.recursive,
            })
            .await?
            .into_inner();
        Ok(resp)
    }

    pub async fn suspend_instance(
        &mut self,
        instance_id: crate::api::InstanceID,
        reason: Option<String>,
    ) -> Result<SuspendResponse> {
        let resp = self
            .client
            .suspend_instance(SuspendRequest {
                instance_id: instance_id.to_string(),
                reason,
            })
            .await?
            .into_inner();
        Ok(resp)
    }

    pub async fn resume_instance(
        &mut self,
        instance_id: crate::api::InstanceID,
        reason: Option<String>,
    ) -> Result<ResumeResponse> {
        let resp = self
            .client
            .resume_instance(ResumeRequest {
                instance_id: instance_id.to_string(),
                reason,
            })
            .await?
            .into_inner();
        Ok(resp)
    }

    pub async fn purge_instances(
        &mut self,
        instance_id: crate::api::InstanceID,
        options: Option<Purge>,
    ) -> Result<PurgeInstancesResponse> {
        let options: Purge = options.expect("failed to parse options");
        let resp = self
            .client
            .purge_instances(PurgeInstancesRequest {
                recursive: options.recursive,
                request: Some(
                    durabletask_proto::purge_instances_request::Request::InstanceId(
                        instance_id.to_string(),
                    ),
                ),
            })
            .await?
            .into_inner();
        Ok(resp)
    }
}
