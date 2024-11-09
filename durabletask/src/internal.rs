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
#![allow(dead_code)]
use std::{any::type_name, borrow::BorrowMut, fmt::Display, str::FromStr, time::SystemTime};

// TODO: remove before publishing crate
use gethostname::gethostname;
use opentelemetry::{
    global::{self, BoxedSpan},
    trace::{
        Span, SpanContext, SpanId, SpanKind, TraceContextExt, TraceFlags, TraceId, TraceState,
        Tracer,
    },
    Context, KeyValue,
};
use prost_wkt_types::Timestamp;
use uuid::Uuid;

use durabletask_proto::{
    history_event::EventType, orchestrator_action::OrchestratorActionType,
    CompleteOrchestrationAction, CreateSubOrchestrationAction, CreateTimerAction, EventRaisedEvent,
    EventSentEvent, ExecutionCompletedEvent, ExecutionResumedEvent, ExecutionStartedEvent,
    ExecutionSuspendedEvent, ExecutionTerminatedEvent, HistoryEvent, OrchestrationInstance,
    OrchestrationStatus, OrchestratorAction, OrchestratorStartedEvent, ParentInstanceInfo,
    ScheduleTaskAction, SendEventAction, SubOrchestrationInstanceCreatedEvent, TaskCompletedEvent,
    TaskFailedEvent, TaskFailureDetails, TaskScheduledEvent, TerminateOrchestrationAction,
    TimerCreatedEvent, TimerFiredEvent, TraceContext,
};

pub(crate) fn new_execution_started_event(
    name: &str,
    instance_id: &str,
    input: Option<&str>,
    parent: Option<ParentInstanceInfo>,
    parent_trace_context: Option<TraceContext>,
    scheduled_start_timestamp: Option<Timestamp>,
) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        event_type: Some(EventType::ExecutionStarted(ExecutionStartedEvent {
            name: name.to_string(),
            parent_instance: parent,
            input: input.map(str::to_string),
            orchestration_instance: Some(OrchestrationInstance {
                instance_id: instance_id.to_string(),
                execution_id: Some(Uuid::new_v4().to_string()),
            }),
            parent_trace_context,
            scheduled_start_timestamp,
            ..Default::default()
        })),
        timestamp: Some(Timestamp::from(SystemTime::now())),
    }
}

pub(crate) fn new_execution_completed_event(
    event_id: i32,
    status: i32,
    result: Option<&str>,
    failure_details: Option<&TaskFailureDetails>,
) -> HistoryEvent {
    HistoryEvent {
        event_id,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::ExecutionCompleted(ExecutionCompletedEvent {
            orchestration_status: status,
            result: result.map(str::to_string),
            failure_details: failure_details.cloned(),
        })),
    }
}

pub(crate) fn new_execution_terminated_event(reason: Option<&str>, recurse: bool) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::ExecutionTerminated(ExecutionTerminatedEvent {
            input: reason.map(str::to_string),
            recurse,
        })),
    }
}

pub(crate) fn new_orchestrator_started_event() -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::OrchestratorStarted(OrchestratorStartedEvent {})),
    }
}

pub(crate) fn new_event_raised_event(name: &str, input: Option<&str>) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::EventRaised(EventRaisedEvent {
            name: name.to_string(),
            input: input.map(str::to_string),
        })),
    }
}

pub(crate) fn new_task_scheduled_event(
    task_id: i32,
    name: &str,
    version: Option<&str>,
    input: Option<&str>,
    trace_context: Option<&TraceContext>,
) -> HistoryEvent {
    HistoryEvent {
        event_id: task_id,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::TaskScheduled(TaskScheduledEvent {
            name: name.to_string(),
            version: version.map(str::to_string),
            input: input.map(str::to_string),
            parent_trace_context: trace_context.cloned(),
        })),
    }
}

pub(crate) fn new_task_completed_event(task_id: i32, result: Option<&str>) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::TaskCompleted(TaskCompletedEvent {
            task_scheduled_id: task_id,
            result: result.map(str::to_string),
        })),
    }
}

pub(crate) fn new_task_failed_event(
    task_id: i32,
    failure_details: Option<&TaskFailureDetails>,
) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::TaskFailed(TaskFailedEvent {
            task_scheduled_id: task_id,
            failure_details: failure_details.cloned(),
        })),
    }
}

pub(crate) fn new_timer_created_event(event_id: i32, fire_at: &Timestamp) -> HistoryEvent {
    HistoryEvent {
        event_id,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::TimerCreated(TimerCreatedEvent {
            fire_at: Some(fire_at.clone()),
        })),
    }
}

pub(crate) fn new_timer_fired_event(timer_id: i32, fire_at: &Timestamp) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::TimerFired(TimerFiredEvent {
            timer_id,
            fire_at: Some(fire_at.clone()),
        })),
    }
}

pub(crate) fn new_sub_orchestration_created_event(
    event_id: i32,
    name: &str,
    version: Option<&str>,
    input: Option<&str>,
    instance_id: &str,
    parent_trace_context: Option<&TraceContext>,
) -> HistoryEvent {
    HistoryEvent {
        event_id,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::SubOrchestrationInstanceCreated(
            SubOrchestrationInstanceCreatedEvent {
                name: name.to_string(),
                version: version.map(str::to_string),
                input: input.map(str::to_string),
                instance_id: instance_id.to_string(),
                parent_trace_context: parent_trace_context.cloned(),
            },
        )),
    }
}

pub(crate) fn new_event_sent_event(
    event_id: i32,
    instance_id: &str,
    name: &str,
    input: Option<&str>,
) -> HistoryEvent {
    HistoryEvent {
        event_id,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::EventSent(EventSentEvent {
            instance_id: instance_id.to_string(),
            name: name.to_string(),
            input: input.map(str::to_string),
        })),
    }
}

pub(crate) fn new_suspend_orchestration_event(reason: Option<&str>) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::ExecutionSuspended(ExecutionSuspendedEvent {
            input: reason.map(str::to_string),
        })),
    }
}

pub(crate) fn new_resume_orchestration_event(reason: Option<&str>) -> HistoryEvent {
    HistoryEvent {
        event_id: -1,
        timestamp: Some(Timestamp::from(SystemTime::now())),
        event_type: Some(EventType::ExecutionResumed(ExecutionResumedEvent {
            input: reason.map(str::to_string),
        })),
    }
}

pub(crate) fn new_parent_info(task_id: i32, name: &str, instance_id: &str) -> ParentInstanceInfo {
    ParentInstanceInfo {
        task_scheduled_id: task_id,
        name: Some(name.to_string()),
        orchestration_instance: Some(OrchestrationInstance {
            instance_id: instance_id.to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(crate) fn new_schedule_task_action(
    task_id: i32,
    name: &str,
    input: Option<&str>,
) -> OrchestratorAction {
    OrchestratorAction {
        id: task_id,
        orchestrator_action_type: Some(OrchestratorActionType::ScheduleTask(ScheduleTaskAction {
            name: name.to_string(),
            input: input.map(str::to_string),
            ..Default::default()
        })),
    }
}

pub(crate) fn new_create_timer_action(task_id: i32, fire_at: &Timestamp) -> OrchestratorAction {
    OrchestratorAction {
        id: task_id,
        orchestrator_action_type: Some(OrchestratorActionType::CreateTimer(CreateTimerAction {
            fire_at: Some(fire_at.clone()),
        })),
    }
}

pub(crate) fn new_send_event_action(
    instance_id: &str,
    name: &str,
    data: Option<&str>,
) -> OrchestratorAction {
    OrchestratorAction {
        id: -1,
        orchestrator_action_type: Some(OrchestratorActionType::SendEvent(SendEventAction {
            instance: Some(OrchestrationInstance {
                instance_id: instance_id.to_string(),
                ..Default::default()
            }),
            name: name.to_string(),
            data: data.map(str::to_string),
        })),
    }
}

pub(crate) fn new_create_sub_orchestration_action(
    task_id: i32,
    name: &str,
    instance_id: &str,
    input: Option<&str>,
) -> OrchestratorAction {
    OrchestratorAction {
        id: task_id,
        orchestrator_action_type: Some(OrchestratorActionType::CreateSubOrchestration(
            CreateSubOrchestrationAction {
                name: name.to_string(),
                input: input.map(str::to_string),
                instance_id: instance_id.to_string(),
                ..Default::default()
            },
        )),
    }
}

pub(crate) fn new_complete_orchestration_action(
    task_id: i32,
    status: OrchestrationStatus,
    result: Option<&str>,
    carryover_events: &[HistoryEvent],
    failure_details: Option<&TaskFailureDetails>,
) -> OrchestratorAction {
    OrchestratorAction {
        id: task_id,
        orchestrator_action_type: Some(OrchestratorActionType::CompleteOrchestration(
            CompleteOrchestrationAction {
                orchestration_status: status as i32,
                result: result.map(str::to_string),
                carryover_events: carryover_events.to_vec(),
                failure_details: failure_details.cloned(),
                ..Default::default()
            },
        )),
    }
}

pub(crate) fn new_terminate_orchestration_action(
    task_id: i32,
    instance_id: &str,
    recurse: bool,
    reason: Option<&str>,
) -> OrchestratorAction {
    OrchestratorAction {
        id: task_id,
        orchestrator_action_type: Some(OrchestratorActionType::TerminateOrchestration(
            TerminateOrchestrationAction {
                instance_id: instance_id.to_string(),
                recurse,
                reason: reason.map(str::to_string),
            },
        )),
    }
}

pub(crate) fn new_task_failure_details<T>(err: T) -> TaskFailureDetails
where
    T: Display,
{
    TaskFailureDetails {
        error_type: type_name::<T>().to_string(),
        error_message: err.to_string(),
        ..Default::default()
    }
}

pub(crate) fn history_list_summary(list: &[HistoryEvent]) -> String {
    let mut sb = String::new();
    sb.push('[');
    for (i, e) in list.iter().enumerate() {
        if i > 0 {
            sb.push_str(", ");
        }
        if i >= 10 {
            sb.push_str("...");
            break;
        }
        let name = get_history_event_type_name(e);
        sb.push_str(&name);
        let task_id = get_task_id(e);
        if task_id >= 0 {
            sb.push('#');
            sb.push_str(&task_id.to_string());
        }
    }
    sb.push(']');
    sb
}

pub(crate) fn action_list_summary(actions: &[OrchestratorAction]) -> String {
    let mut sb = String::new();
    sb.push('[');
    for (i, a) in actions.iter().enumerate() {
        if i > 0 {
            sb.push_str(", ");
        }
        if i >= 10 {
            sb.push_str("...");
            break;
        }
        let name = get_action_type_name(a);
        sb.push_str(&name);
        if a.id >= 0 {
            sb.push('#');
            sb.push_str(&a.id.to_string());
        }
    }
    sb.push(']');
    sb
}

pub(crate) fn get_task_id(e: &HistoryEvent) -> i32 {
    if e.event_id != 0 {
        e.event_id
    } else {
        match &e.event_type {
            None => -1,
            Some(EventType::TaskCompleted(t)) => t.task_scheduled_id,
            Some(EventType::TaskFailed(t)) => t.task_scheduled_id,
            Some(EventType::SubOrchestrationInstanceCompleted(so)) => so.task_scheduled_id,
            Some(EventType::SubOrchestrationInstanceFailed(so)) => so.task_scheduled_id,
            Some(EventType::TimerFired(t)) => t.timer_id,
            Some(EventType::ExecutionStarted(ex)) => {
                ex.clone().parent_instance.unwrap().task_scheduled_id
            }
            Some(_) => -1,
        }
    }
}

pub(crate) fn to_runtime_status_string(status: OrchestrationStatus) -> String {
    status.as_str_name()[21..].to_string()
}

pub(crate) fn from_runtime_status_string(status: &str) -> OrchestrationStatus {
    let runtime_status = format!("ORCHESTRATION_STATUS_{}", status);
    OrchestrationStatus::from_str_name(&runtime_status).unwrap()
}

pub(crate) fn get_history_event_type_name(e: &HistoryEvent) -> String {
    if e.event_type.is_none() {
        "".to_string()
    } else {
        let event_type = match e.event_type {
            Some(EventType::ExecutionStarted(_)) => "ExecutionStarted",
            Some(EventType::ExecutionResumed(_)) => "ExecutionResumed",
            Some(EventType::ExecutionCompleted(_)) => "ExecutionCompleted",
            Some(EventType::ExecutionSuspended(_)) => "ExecutionSuspended",
            Some(EventType::ExecutionTerminated(_)) => "ExecutionTerminated",

            Some(EventType::TimerFired(_)) => "TimerFired",
            Some(EventType::TimerCreated(_)) => "TimerCreated",

            Some(EventType::TaskScheduled(_)) => "TaskScheduled",
            Some(EventType::TaskFailed(_)) => "TaskFailed",
            Some(EventType::TaskCompleted(_)) => "TaskCompleted",

            Some(EventType::SubOrchestrationInstanceFailed(_)) => "SubOrchestrationInstanceFailed",
            Some(EventType::SubOrchestrationInstanceCompleted(_)) => {
                "SubOrchestrationInstanceFailed"
            }
            Some(EventType::SubOrchestrationInstanceCreated(_)) => {
                "SubOrchestrationInstanceCreated"
            }

            Some(EventType::OrchestratorStarted(_)) => "OrchestratorStarted",
            Some(EventType::OrchestratorCompleted(_)) => "OrchestratorCompleted",

            Some(EventType::EventSent(_)) => "EventSent",
            Some(EventType::EventRaised(_)) => "EventRaised",

            Some(EventType::GenericEvent(_)) => "GenericEvent",

            Some(EventType::HistoryState(_)) => "HistoryEvent",

            Some(EventType::ContinueAsNew(_)) => "ContinueAsNew",

            None => "",
        };
        event_type.to_string()
    }
}

pub(crate) fn get_action_type_name(a: &OrchestratorAction) -> String {
    if a.orchestrator_action_type.is_none() {
        "".to_string()
    } else {
        let action_type = match a.orchestrator_action_type {
            Some(OrchestratorActionType::SendEvent(_)) => "SendEvent",
            Some(OrchestratorActionType::CreateTimer(_)) => "CreateTimer",
            Some(OrchestratorActionType::ScheduleTask(_)) => "ScheduleTask",
            Some(OrchestratorActionType::CompleteOrchestration(_)) => "CompleteOrchestration",
            Some(OrchestratorActionType::CreateSubOrchestration(_)) => "CreateSubOrchestration",
            Some(OrchestratorActionType::TerminateOrchestration(_)) => "TerminateOrchestration",

            None => "",
        };
        action_type.to_string()
    }
}

pub(crate) fn start_new_create_orchestration_span(
    ctx: &Context,
    name: &'static str,
    version: &'static str,
    instance_id: &'static str,
) -> BoxedSpan {
    let attributes = vec![
        KeyValue::new("durabletask.type", "orchestration"),
        KeyValue::new("durabletask.task.name", name),
        KeyValue::new("durabletask.task.instance_id", instance_id),
    ];
    start_new_span(
        ctx.clone(),
        "create_orchestration",
        name,
        version,
        attributes,
        SpanKind::Client,
        SystemTime::now(),
    )
}

pub(crate) fn start_new_run_orchestration_span(
    ctx: Context,
    es: &'static ExecutionStartedEvent,
    started_time: SystemTime,
) -> BoxedSpan {
    let name = &es.name;
    let instance_id = es.orchestration_instance.clone().unwrap().instance_id;
    let version: &str = es.version.as_ref().unwrap();
    let attributes = vec![
        KeyValue::new("durabletask.type", "orchestration"),
        KeyValue::new("durabletask.task.name", name.clone()),
        KeyValue::new("durabletask.task.instance_id", instance_id),
    ];
    start_new_span(
        ctx,
        "orchestration",
        name,
        version,
        attributes,
        SpanKind::Server,
        started_time,
    )
}

pub(crate) fn start_new_activity_span(
    ctx: Context,
    name: &'static str,
    version: &'static str,
    instance_id: &'static str,
    task_id: i32,
) -> BoxedSpan {
    let attributes = vec![
        KeyValue::new("durabletask.type", "activity"),
        KeyValue::new("durabletask.task.name", name),
        KeyValue::new("durabletask.task.task_id", task_id.to_string()),
        KeyValue::new("durabletask.task.instance_id", instance_id),
    ];
    start_new_span(
        ctx,
        "activity",
        name,
        version,
        attributes,
        SpanKind::Server,
        SystemTime::now(),
    )
}

pub(crate) fn start_and_end_new_timer_span(
    ctx: Context,
    tf: &TimerFiredEvent,
    created_time: SystemTime,
    instance_id: &'static str,
) -> Result<(), ()> {
    let attributes = vec![
        KeyValue::new("durabletask.type", "timer"),
        KeyValue::new("durabletask.fire_at", tf.fire_at.as_ref().unwrap().seconds),
        KeyValue::new("durabletask.task.task_id", tf.timer_id.to_string()),
        KeyValue::new("durabletask.task.instance_id", instance_id),
    ];

    let mut span = start_new_span(
        ctx,
        "timer",
        "",
        "",
        attributes,
        SpanKind::Internal,
        created_time,
    );
    span.end();
    Ok(())
}

pub(crate) fn start_new_span(
    ctx: Context,
    task_type: &str,
    task_name: &str,
    task_version: &'static str,
    attributes: Vec<KeyValue>,
    kind: SpanKind,
    timestamp: SystemTime,
) -> BoxedSpan {
    let span_name = if !task_version.is_empty() {
        format!("{}||{}||{}", task_type, task_name, task_version)
    } else if !task_name.is_empty() {
        format!("{}||{}", task_type, task_name)
    } else {
        task_type.to_string()
    };

    let attributes = match task_version.is_empty() {
        true => attributes,
        false => {
            let mut attributes = attributes;
            attributes.push(KeyValue::new("durabletask.task.version", task_version));
            attributes
        }
    };
    let tracer = global::tracer("durabletask");

    let span_builder = tracer
        .span_builder(span_name)
        .with_kind(kind)
        .with_start_time(timestamp)
        .with_attributes(attributes);

    span_builder.start_with_context(&tracer, &ctx)
}

pub(crate) fn unsafe_set_span_context(span: BoxedSpan, span_context: SpanContext) -> bool {
    if !span.is_recording() {
        return false;
    }

    *span.span_context().borrow_mut() = &span_context;
    true
}

fn context_from_trace_context(ctx: &Context, tc: &TraceContext) -> Result<Context, ()> {
    if tc.trace_state.is_none() {
        return Ok(ctx.clone());
    }

    let span_context = span_context_from_trace_context(tc)?;
    Ok(ctx.with_remote_span_context(span_context))
}

pub(crate) fn span_context_from_trace_context(tc: &TraceContext) -> Result<SpanContext, ()> {
    let trace_parent = &tc.trace_parent;
    let parts: Vec<_> = trace_parent.split('-').collect();
    if parts.len() != 4 {
        // backwards compatibility with older versions of the protobuf
        let trace_id = TraceId::from_hex(&tc.trace_parent).map_err(|_| ())?;
        #[allow(deprecated)]
        let span_id = SpanId::from_hex(&tc.span_id).map_err(|_| ())?;
        let trace_flags: TraceFlags =
            TraceFlags::new(u8::from_str_radix(parts.get(3).unwrap(), 16).map_err(|_| ())?);
        let span_context_config =
            SpanContext::new(trace_id, span_id, trace_flags, true, TraceState::default());
        return Ok(span_context_config);
    }

    let trace_id = TraceId::from_hex(parts[1]).map_err(|_| ())?;
    let span_id = SpanId::from_hex(parts[2]).map_err(|_| ())?;
    let trace_flags: TraceFlags =
        TraceFlags::new(u8::from_str_radix(parts[3], 16).map_err(|_| ())?);

    let span_context_config = match &tc.trace_state {
        Some(trace_state) => SpanContext::new(
            trace_id,
            span_id,
            trace_flags,
            true,
            TraceState::from_str(trace_state).unwrap(),
        ),
        None => SpanContext::new(trace_id, span_id, trace_flags, true, TraceState::default()),
    };

    Ok(span_context_config)
}

pub(crate) fn trace_context_from_span(span: Option<BoxedSpan>) -> Option<TraceContext> {
    if span.is_none() || !span.as_ref().unwrap().span_context().is_sampled() {
        return None;
    }

    let span_context = span.as_ref().unwrap().span_context();
    let trace_parent = format!(
        "00-{}-{}-{}",
        span_context.trace_id(),
        span_context.span_id(),
        span_context.trace_flags().to_u8(),
    );

    let trace_state: Option<String> = if !span_context.trace_state().header().is_empty() {
        // TODO: refactor this disgusting block into a match
        Some(span_context.trace_state().header())
    } else {
        None
    };
    let tc = TraceContext {
        trace_parent,
        trace_state,
        ..Default::default()
    };

    Some(tc)
}

pub(crate) fn change_span_id(span: BoxedSpan, new_span_id: SpanId) {
    let modified_span_context = span.span_context().borrow_mut().clone();
    *modified_span_context.span_id().borrow_mut() = new_span_id;
    unsafe_set_span_context(span, modified_span_context);
}

pub(crate) fn cancel_span(span: BoxedSpan) {
    if span.span_context().is_sampled() {
        let modified_span_context = span.span_context().borrow_mut().clone();
        *modified_span_context.trace_flags().borrow_mut() = TraceFlags::default();
        unsafe_set_span_context(span, modified_span_context);
    }
}

pub(crate) fn noop_span() -> BoxedSpan {
    global::tracer("durabletask").start("noop")
}

/// Return the function name as a String
pub(crate) fn get_task_function_name<F>(_: F) -> String {
    let function_path = std::any::type_name::<F>();
    // Return the function name without the preceding path
    function_path.split("::").last().unwrap().to_string()
}

/// Return the default worker name consisting of:
///
/// - Hostname
/// - Process ID (PID)
/// - Universally Unique Identifier v4 (UUID)
///
/// Returned as a String in this format:
/// {Hostname},{PID},{UUID}
pub(crate) fn get_default_worker_name() -> String {
    let hostname: String = gethostname().into_string().unwrap_or("unknown".to_string());
    let pid = std::process::id();
    let uuid = Uuid::new_v4();
    format!("{hostname},{pid},{uuid}")
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::internal::get_default_worker_name;

    use super::get_task_function_name;

    #[test]
    fn test_get_task_function_name() {
        fn test_task_function(output: String) {
            println!("finished something: {output}")
        }
        let result = get_task_function_name(test_task_function);
        assert_eq!(result, "test_task_function")
    }

    #[test]
    fn test_get_default_worker_name() {
        let result = get_default_worker_name();
        println!("{}", result.clone()); //debug
        let parsed: Vec<String> = result.split(',').map(|s| s.to_string()).collect();
        assert_ne!(parsed[0], "unknown");
        assert!(parsed[1].parse::<u64>().is_ok());
        let id = Uuid::parse_str(&parsed[2]).unwrap();
        assert_eq!(id.get_version(), Some(uuid::Version::Random));
    }
}
