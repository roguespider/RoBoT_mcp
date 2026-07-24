// src/tools/workflow.rs
//! Workflow-related MCP tools - create, manage, and execute workflows

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::tools::ToolOutput;
use crate::workflows::engine::{Workflow, WorkflowEngine, WorkflowStatus};

/// Tool: Create a new workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateWorkflowInput {
    pub name: String,
    pub description: Option<String>,
}

/// Tool: Add a step to a workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddWorkflowStepInput {
    pub workflow_id: String,
    pub name: String,
    pub action: String,
    pub parameters: Option<String>,
}

/// Tool: Get workflow status/details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWorkflowStatusInput {
    pub workflow_id: String,
}

/// Tool: List all workflows
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListWorkflowsInput {
    pub status: Option<String>,
}

/// Tool: Start a workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StartWorkflowInput {
    pub workflow_id: String,
}

/// Tool: Pause a workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PauseWorkflowInput {
    pub workflow_id: String,
}

/// Tool: Resume a paused workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResumeWorkflowInput {
    pub workflow_id: String,
}

/// Tool: Cancel a workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CancelWorkflowInput {
    pub workflow_id: String,
}

/// Tool: Delete a workflow
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteWorkflowInput {
    pub workflow_id: String,
}

/// Workflow tool definitions
pub mod definitions {
    pub const CREATE_WORKFLOW: &str = "create_workflow";
    pub const ADD_WORKFLOW_STEP: &str = "add_workflow_step";
    pub const GET_WORKFLOW_STATUS: &str = "get_workflow_status";
    pub const LIST_WORKFLOWS: &str = "list_workflows";
    pub const START_WORKFLOW: &str = "start_workflow";
    pub const PAUSE_WORKFLOW: &str = "pause_workflow";
    pub const RESUME_WORKFLOW: &str = "resume_workflow";
    pub const CANCEL_WORKFLOW: &str = "cancel_workflow";
    pub const DELETE_WORKFLOW: &str = "delete_workflow";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: CREATE_WORKFLOW.to_string(),
                description: "Create a new workflow with a name and optional description"
                    .to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the workflow"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional description of the workflow"
                        }
                    },
                    "required": ["name"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_WORKFLOW_STEP.to_string(),
                description: "Add a step to an existing workflow. Steps are executed in order."
                    .to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to add the step to"
                        },
                        "name": {
                            "type": "string",
                            "description": "Human-readable name for this step"
                        },
                        "action": {
                            "type": "string",
                            "description": "The action to perform (e.g., 'store_memory', 'search_memory', 'record_experience')"
                        },
                        "parameters": {
                            "type": "string",
                            "description": "Optional JSON string of parameters for the action"
                        }
                    },
                    "required": ["workflow_id", "name", "action"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_WORKFLOW_STATUS.to_string(),
                description: "Get the current status and details of a workflow".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to query"
                        }
                    },
                    "required": ["workflow_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_WORKFLOWS.to_string(),
                description: "List all workflows, optionally filtered by status".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Filter by status: draft, ready, running, paused, completed, failed, cancelled",
                            "enum": ["draft", "ready", "running", "paused", "completed", "failed", "cancelled"]
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: START_WORKFLOW.to_string(),
                description:
                    "Start executing a workflow. The engine will run all steps sequentially."
                        .to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to start"
                        }
                    },
                    "required": ["workflow_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: PAUSE_WORKFLOW.to_string(),
                description: "Pause a running workflow".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to pause"
                        }
                    },
                    "required": ["workflow_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: RESUME_WORKFLOW.to_string(),
                description: "Resume a paused workflow".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to resume"
                        }
                    },
                    "required": ["workflow_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: CANCEL_WORKFLOW.to_string(),
                description: "Cancel a workflow, removing it from execution.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to cancel"
                        }
                    },
                    "required": ["workflow_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: DELETE_WORKFLOW.to_string(),
                description: "Delete a workflow completely.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "ID of the workflow to delete"
                        }
                    },
                    "required": ["workflow_id"]
                }),
            },
        ]
    }
}

/// Convert WorkflowStatus to string for JSON
fn status_to_string(s: WorkflowStatus) -> &'static str {
    match s {
        WorkflowStatus::Draft => "draft",
        WorkflowStatus::Ready => "ready",
        WorkflowStatus::Running => "running",
        WorkflowStatus::Paused => "paused",
        WorkflowStatus::Completed => "completed",
        WorkflowStatus::Failed => "failed",
        WorkflowStatus::Cancelled => "cancelled",
    }
}

fn workflow_to_json(w: &Workflow) -> serde_json::Value {
    serde_json::json!({
        "id": w.id,
        "name": w.name,
        "description": w.description,
        "status": status_to_string(w.status),
        "steps": w.steps.iter().map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "action": s.action,
            "parameters": s.parameters,
            "retry_count": s.retry_count,
            "max_retries": s.max_retries,
            "timeout_seconds": s.timeout_seconds,
            "on_success": s.on_success,
            "on_failure": s.on_failure,
        })).collect::<Vec<_>>(),
        "variables": w.variables,
        "created_at": w.created_at.to_rfc3339(),
        "started_at": w.started_at.map(|t| t.to_rfc3339()),
        "completed_at": w.completed_at.map(|t| t.to_rfc3339()),
    })
}

/// Execute create workflow tool
pub async fn execute_create_workflow(
    input: CreateWorkflowInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    let description = input.description.unwrap_or_default();
    let workflow = engine.create_workflow(input.name, description).await;
    ToolOutput::success(serde_json::json!({
        "success": true,
        "message": "Workflow created",
        "workflow": workflow_to_json(&workflow),
    }))
}

/// Execute add workflow step tool
pub async fn execute_add_workflow_step(
    input: AddWorkflowStepInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine
        .add_step(&input.workflow_id, input.name, input.action)
        .await
    {
        Ok(Some(step)) => ToolOutput::success(serde_json::json!({
            "success": true,
            "message": "Step added",
            "step": {
                "id": step.id,
                "name": step.name,
                "action": step.action,
                "max_retries": step.max_retries,
                "timeout_seconds": step.timeout_seconds,
            }
        })),
        Ok(None) => ToolOutput::success(serde_json::json!({
            "success": false,
            "message": format!("Workflow '{}' not found", input.workflow_id),
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute get workflow status tool
pub async fn execute_get_workflow_status(
    input: GetWorkflowStatusInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine.get_workflow(&input.workflow_id).await {
        Some(workflow) => ToolOutput::success(serde_json::json!({
            "success": true,
            "found": true,
            "workflow": workflow_to_json(&workflow),
        })),
        None => ToolOutput::success(serde_json::json!({
            "success": true,
            "found": false,
            "message": format!("Workflow '{}' not found", input.workflow_id),
        })),
    }
}

/// Execute list workflows tool
pub async fn execute_list_workflows(
    input: ListWorkflowsInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    let workflows = if let Some(ref status) = input.status {
        let status_lower = status.to_lowercase();
        let status_enum = match status_lower.as_str() {
            "draft" => WorkflowStatus::Draft,
            "ready" => WorkflowStatus::Ready,
            "running" => WorkflowStatus::Running,
            "paused" => WorkflowStatus::Paused,
            "completed" => WorkflowStatus::Completed,
            "failed" => WorkflowStatus::Failed,
            "cancelled" => WorkflowStatus::Cancelled,
            _ => return ToolOutput::error(format!("Invalid status: {}", status)),
        };
        engine.list_by_status(status_enum).await
    } else {
        engine.list_workflows().await
    };

    ToolOutput::success(serde_json::json!({
        "success": true,
        "workflows": workflows.iter().map(workflow_to_json).collect::<Vec<_>>(),
        "count": workflows.len(),
    }))
}

/// Execute start workflow tool
pub async fn execute_start_workflow(
    input: StartWorkflowInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine.start(&input.workflow_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "success": true,
            "message": format!("Workflow '{}' started", input.workflow_id),
            "workflow_id": input.workflow_id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute pause workflow tool
pub async fn execute_pause_workflow(
    input: PauseWorkflowInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine.pause(&input.workflow_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "success": true,
            "message": format!("Workflow '{}' paused", input.workflow_id),
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute resume workflow tool
pub async fn execute_resume_workflow(
    input: ResumeWorkflowInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine.resume(&input.workflow_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "success": true,
            "message": format!("Workflow '{}' resumed", input.workflow_id),
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute cancel workflow tool
pub async fn execute_cancel_workflow(
    input: CancelWorkflowInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine.cancel(&input.workflow_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "success": true,
            "message": format!("Workflow '{}' cancelled", input.workflow_id),
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute delete workflow tool
pub async fn execute_delete_workflow(
    input: DeleteWorkflowInput,
    engine: &Arc<WorkflowEngine>,
) -> ToolOutput {
    match engine.delete(&input.workflow_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "success": true,
            "message": format!("Workflow '{}' deleted", input.workflow_id),
        })),
        Err(e) => ToolOutput::error(e),
    }
}
