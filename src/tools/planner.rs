// src/tools/planner.rs
//! Planner MCP tools - task decomposition and execution

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::planner::planner::{Planner, Plan, PlanStatus};
use crate::tools::ToolOutput;

/// Tool: Create a new plan
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreatePlanInput {
    pub goal: String,
}

/// Tool: Add a step to a plan
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddPlanStepInput {
    pub plan_id: String,
    pub description: String,
    pub action: String,
}

/// Tool: Add dependency to a step
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddStepDependencyInput {
    pub plan_id: String,
    pub step_id: String,
    pub depends_on: String,
}

/// Tool: Get a plan by ID
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetPlanInput {
    pub plan_id: String,
}

/// Tool: List active plans
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListPlansInput {
    pub status: Option<String>,
}

/// Tool: Start executing a plan
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StartPlanInput {
    pub plan_id: String,
}

/// Tool: Complete a step
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CompleteStepInput {
    pub plan_id: String,
    pub step_id: String,
    pub result: Option<String>,
}

/// Tool: Fail a step
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FailStepInput {
    pub plan_id: String,
    pub step_id: String,
    pub error: String,
}

/// Tool: Cancel a plan
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CancelPlanInput {
    pub plan_id: String,
}

/// Planner tool definitions
pub mod definitions {
    pub const CREATE_PLAN: &str = "create_plan";
    pub const ADD_PLAN_STEP: &str = "add_plan_step";
    pub const ADD_STEP_DEPENDENCY: &str = "add_step_dependency";
    pub const GET_PLAN: &str = "get_plan";
    pub const LIST_PLANS: &str = "list_plans";
    pub const START_PLAN: &str = "start_plan";
    pub const COMPLETE_STEP: &str = "complete_step";
    pub const FAIL_STEP: &str = "fail_step";
    pub const CANCEL_PLAN: &str = "cancel_plan";
    
    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: CREATE_PLAN.to_string(),
                description: "Create a new plan from a goal".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal": {
                            "type": "string",
                            "description": "The goal to plan for"
                        }
                    },
                    "required": ["goal"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_PLAN_STEP.to_string(),
                description: "Add a step to an existing plan".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan to add the step to"
                        },
                        "description": {
                            "type": "string",
                            "description": "Human-readable description of the step"
                        },
                        "action": {
                            "type": "string",
                            "description": "The action to perform"
                        }
                    },
                    "required": ["plan_id", "description", "action"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_STEP_DEPENDENCY.to_string(),
                description: "Add a dependency between steps".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan"
                        },
                        "step_id": {
                            "type": "string",
                            "description": "Step that depends on another"
                        },
                        "depends_on": {
                            "type": "string",
                            "description": "ID of the step this step depends on"
                        }
                    },
                    "required": ["plan_id", "step_id", "depends_on"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_PLAN.to_string(),
                description: "Get a plan by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan to retrieve"
                        }
                    },
                    "required": ["plan_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_PLANS.to_string(),
                description: "List all active plans".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Filter by status: pending, in_progress, completed, failed, cancelled"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: START_PLAN.to_string(),
                description: "Start executing a plan".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan to start"
                        }
                    },
                    "required": ["plan_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COMPLETE_STEP.to_string(),
                description: "Mark a step as completed".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan"
                        },
                        "step_id": {
                            "type": "string",
                            "description": "ID of the step to complete"
                        },
                        "result": {
                            "type": "string",
                            "description": "Optional result of the step"
                        }
                    },
                    "required": ["plan_id", "step_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: FAIL_STEP.to_string(),
                description: "Mark a step as failed".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan"
                        },
                        "step_id": {
                            "type": "string",
                            "description": "ID of the step that failed"
                        },
                        "error": {
                            "type": "string",
                            "description": "Error message"
                        }
                    },
                    "required": ["plan_id", "step_id", "error"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: CANCEL_PLAN.to_string(),
                description: "Cancel a plan".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {
                            "type": "string",
                            "description": "ID of the plan to cancel"
                        }
                    },
                    "required": ["plan_id"]
                }),
            },
        ]
    }
}

fn plan_to_json(plan: &Plan) -> serde_json::Value {
    serde_json::json!({
        "id": plan.id,
        "goal": plan.goal,
        "status": format!("{:?}", plan.status),
        "steps": plan.steps.iter().map(|s| serde_json::json!({
            "id": s.id,
            "description": s.description,
            "action": s.action,
            "dependencies": s.dependencies,
            "status": format!("{:?}", s.status),
            "result": s.result,
        })).collect::<Vec<_>>(),
        "created_at": plan.created_at.to_rfc3339(),
        "completed_at": plan.completed_at.map(|t| t.to_rfc3339()),
    })
}

/// Execute create plan
pub async fn execute_create_plan(
    input: CreatePlanInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.create_plan(&input.goal).await {
        Ok(plan) => ToolOutput::success(serde_json::json!({
            "status": "created",
            "plan": plan_to_json(&plan),
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute add plan step
pub async fn execute_add_plan_step(
    input: AddPlanStepInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.add_step(&input.plan_id, &input.description, &input.action).await {
        Ok(step) => ToolOutput::success(serde_json::json!({
            "status": "added",
            "step": {
                "id": step.id,
                "description": step.description,
                "action": step.action,
                "status": format!("{:?}", step.status),
            }
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute add step dependency
pub async fn execute_add_step_dependency(
    input: AddStepDependencyInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.add_dependency(&input.plan_id, &input.step_id, &input.depends_on).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "added",
            "message": format!("Step {} now depends on {}", input.step_id, input.depends_on),
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute get plan
pub async fn execute_get_plan(
    input: GetPlanInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.get_plan(&input.plan_id).await {
        Some(plan) => ToolOutput::success(serde_json::json!({
            "found": true,
            "plan": plan_to_json(&plan),
        })),
        None => ToolOutput::success(serde_json::json!({
            "found": false,
            "message": format!("Plan {} not found", input.plan_id),
        })),
    }
}

/// Execute list plans
pub async fn execute_list_plans(
    input: ListPlansInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    let plans = planner.list_plans().await;
    
    let filtered: Vec<_> = if let Some(status) = input.status {
        let status_lower = status.to_lowercase();
        plans.into_iter().filter(|p| {
            let s = match status_lower.as_str() {
                "pending" => PlanStatus::Pending,
                "in_progress" | "inprogress" => PlanStatus::InProgress,
                "completed" => PlanStatus::Completed,
                "failed" => PlanStatus::Failed,
                "cancelled" => PlanStatus::Cancelled,
                _ => return true,
            };
            p.status == s
        }).collect()
    } else {
        plans
    };
    
    let result: Vec<_> = filtered.iter().map(plan_to_json).collect();
    
    ToolOutput::success(serde_json::json!({
        "plans": result,
        "count": result.len(),
    }))
}

/// Execute start plan
pub async fn execute_start_plan(
    input: StartPlanInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.start_plan(&input.plan_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "started",
            "plan_id": input.plan_id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute complete step
pub async fn execute_complete_step(
    input: CompleteStepInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.complete_step(&input.plan_id, &input.step_id, input.result).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "completed",
            "plan_id": input.plan_id,
            "step_id": input.step_id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute fail step
pub async fn execute_fail_step(
    input: FailStepInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.fail_step(&input.plan_id, &input.step_id, input.error).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "failed",
            "plan_id": input.plan_id,
            "step_id": input.step_id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute cancel plan
pub async fn execute_cancel_plan(
    input: CancelPlanInput,
    planner: &Arc<Planner>,
) -> ToolOutput {
    match planner.cancel_plan(&input.plan_id).await {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "cancelled",
            "plan_id": input.plan_id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}
