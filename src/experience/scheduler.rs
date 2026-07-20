// /src/experience/scheduler.rs
// Background job scheduler for learning tasks

use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// A scheduled task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique identifier
    pub id: String,

    /// Task name
    pub name: String,

    /// Task type
    pub task_type: TaskType,

    /// Scheduling configuration
    pub schedule: TaskSchedule,

    /// Current status
    pub status: TaskStatus,

    /// Last execution time
    pub last_run: Option<DateTime<Utc>>,

    /// Next scheduled execution
    pub next_run: Option<DateTime<Utc>>,

    /// Number of consecutive failures
    pub failure_count: u32,

    /// Creation time
    pub created_at: DateTime<Utc>,
}

/// Type of task to execute
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Run reflection on recent experiences
    Reflection,
    
    /// Run hypothesis evaluation
    HypothesisEvaluation,
    
    /// Run exploration analysis
    ExplorationAnalysis,
    
    /// Clean up old data
    Cleanup,
    
    /// Run metrics collection
    MetricsCollection,
    
    /// Run evolution maintenance
    EvolutionMaintenance,
    
    /// Run reputation decay
    ReputationDecay,
    
    /// Custom task
    Custom,
}

/// How the task is scheduled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSchedule {
    /// Run at fixed interval (seconds)
    Interval { seconds: u64 },
    
    /// Run at specific times (cron-like, hour:minute)
    Daily { hour: u8, minute: u8 },
    
    /// Run on specific days of week (0=Sunday, 6=Saturday)
    Weekly { day: u8, hour: u8, minute: u8 },
    
    /// Run once at specific time
    Once { at: DateTime<Utc> },
    
    /// Manual trigger only
    Manual,
}

/// Task execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Scheduled and waiting
    Scheduled,
    
    /// Currently running
    Running,
    
    /// Completed successfully
    Completed,
    
    /// Failed to execute
    Failed,
    
    /// Disabled by user
    Disabled,
}

/// Scheduler for background tasks
pub struct Scheduler {
    database: Arc<SqliteDatabase>,
    task_handlers: Arc<RwLock<std::collections::HashMap<TaskType, Box<dyn TaskHandler>>>>,
}

impl Scheduler {
    /// Create a new scheduler with database
    pub fn new(database: Arc<SqliteDatabase>) -> Self {
        Self {
            database,
            task_handlers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Load tasks from database
    pub async fn load_tasks(&self) -> Result<Vec<ScheduledTask>> {
        let conn = self.database.connection()?;
        let tasks = queries::list_scheduled_tasks(&conn)?;
        Ok(tasks)
    }

    /// Schedule a new task
    pub async fn schedule(&self, task: ScheduledTask) -> Result<String> {
        let id = task.id.clone();
        let conn = self.database.connection()?;
        queries::insert_scheduled_task(&conn, &task)?;
        tracing::info!("Scheduled task: {}", id);
        Ok(id)
    }

    /// Create and schedule a task
    pub async fn create_task(
        &self,
        name: impl Into<String>,
        task_type: TaskType,
        schedule: TaskSchedule,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let next_run = Self::calculate_next_run(&schedule);
        
        let task = ScheduledTask {
            id: id.clone(),
            name: name.into(),
            task_type,
            schedule,
            status: TaskStatus::Scheduled,
            last_run: None,
            next_run,
            failure_count: 0,
            created_at: Utc::now(),
        };
        
        self.schedule(task).await
    }

    /// Get a task by ID
    pub async fn get_task(&self, id: &str) -> Result<Option<ScheduledTask>> {
        let conn = self.database.connection()?;
        queries::get_scheduled_task(&conn, id)
    }

    /// Get all tasks
    pub async fn list_tasks(&self) -> Result<Vec<ScheduledTask>> {
        let conn = self.database.connection()?;
        queries::list_scheduled_tasks(&conn)
    }

    /// Get tasks ready to run
    pub async fn get_due_tasks(&self) -> Result<Vec<ScheduledTask>> {
        let now = Utc::now();
        let conn = self.database.connection()?;
        let all_tasks = queries::list_scheduled_tasks(&conn)?;
        Ok(all_tasks
            .into_iter()
            .filter(|t| {
                t.status == TaskStatus::Scheduled
                    && t.next_run.map(|n| n <= now).unwrap_or(false)
            })
            .collect())
    }

    /// Mark task as running
    pub async fn start_task(&self, id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        if let Some(mut task) = queries::get_scheduled_task(&conn, id)? {
            task.status = TaskStatus::Running;
            queries::insert_scheduled_task(&conn, &task)?;
        }
        Ok(())
    }

    /// Mark task as completed
    pub async fn complete_task(&self, id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        if let Some(mut task) = queries::get_scheduled_task(&conn, id)? {
            task.status = TaskStatus::Completed;
            task.last_run = Some(Utc::now());
            task.next_run = Self::calculate_next_run(&task.schedule);
            task.failure_count = 0;
            queries::insert_scheduled_task(&conn, &task)?;
        }
        Ok(())
    }

    /// Mark task as failed
    pub async fn fail_task(&self, id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        if let Some(mut task) = queries::get_scheduled_task(&conn, id)? {
            task.status = TaskStatus::Failed;
            task.failure_count += 1;
            if task.failure_count >= 5 {
                task.status = TaskStatus::Disabled;
                tracing::warn!("Task {} disabled after {} failures", id, task.failure_count);
            }
            queries::insert_scheduled_task(&conn, &task)?;
        }
        Ok(())
    }

    /// Cancel (disable) a task
    pub async fn cancel_task(&self, id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        if let Some(mut task) = queries::get_scheduled_task(&conn, id)? {
            task.status = TaskStatus::Disabled;
            queries::insert_scheduled_task(&conn, &task)?;
        }
        Ok(())
    }

    /// Re-enable a disabled task
    pub async fn enable_task(&self, id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        if let Some(mut task) = queries::get_scheduled_task(&conn, id)? {
            if task.status == TaskStatus::Disabled {
                task.status = TaskStatus::Scheduled;
                task.failure_count = 0;
                task.next_run = Self::calculate_next_run(&task.schedule);
                queries::insert_scheduled_task(&conn, &task)?;
            }
        }
        Ok(())
    }

    /// Delete a task
    pub async fn delete_task(&self, id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        queries::delete_scheduled_task(&conn, id)?;
        tracing::info!("Deleted task: {}", id);
        Ok(())
    }

    /// Register a handler for a task type
    pub async fn register_handler(&self, task_type: TaskType, handler: Box<dyn TaskHandler>) {
        let mut handlers = self.task_handlers.write().await;
        handlers.insert(task_type, handler);
    }

    /// Execute a task by ID
    pub async fn execute_task(&self, id: &str) -> Result<()> {
        let task = self.get_task(id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", id))?;
        
        let handlers = self.task_handlers.read().await;
        let handler = handlers.get(&task.task_type)
            .ok_or_else(|| anyhow::anyhow!("No handler for task type: {:?}", task.task_type))?;
        
        self.start_task(id).await?;
        
        match handler.execute().await {
            Ok(()) => {
                self.complete_task(id).await?;
            }
            Err(e) => {
                tracing::error!("Task {} failed: {}", id, e);
                self.fail_task(id).await?;
            }
        }
        
        Ok(())
    }

    /// Calculate next run time based on schedule
    fn calculate_next_run(schedule: &TaskSchedule) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        
        match schedule {
            TaskSchedule::Interval { seconds } => {
                Some(now + chrono::Duration::seconds(*seconds as i64))
            }
            TaskSchedule::Daily { hour, minute } => {
                let today = now.date_naive();
                let scheduled = today.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                let scheduled: DateTime<Utc> = scheduled.and_utc();
                if scheduled <= now {
                    Some(scheduled + chrono::Duration::days(1))
                } else {
                    Some(scheduled)
                }
            }
            TaskSchedule::Weekly { day, hour, minute } => {
                let current_day = now.weekday().num_days_from_sunday() as u8;
                let days_until = if *day >= current_day {
                    (*day - current_day) as i64
                } else {
                    (7 - current_day as i64) + *day as i64
                };
                
                let next_date = now.date_naive() + chrono::Duration::days(days_until);
                let scheduled = next_date.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                Some(scheduled.and_utc())
            }
            TaskSchedule::Once { at } => {
                if *at > now {
                    Some(*at)
                } else {
                    None
                }
            }
            TaskSchedule::Manual => None,
        }
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> Result<SchedulerStats> {
        let conn = self.database.connection()?;
        let tasks = queries::list_scheduled_tasks(&conn)?;
        let total = tasks.len();
        
        let by_status: std::collections::HashMap<TaskStatus, usize> = tasks
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, t| {
                *acc.entry(t.status).or_insert(0) += 1;
                acc
            });

        let by_type: std::collections::HashMap<TaskType, usize> = tasks
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, t| {
                *acc.entry(t.task_type).or_insert(0) += 1;
                acc
            });

        let total_failures: u32 = tasks.iter().map(|t| t.failure_count).sum();

        Ok(SchedulerStats {
            total_tasks: total,
            tasks_by_status: by_status,
            tasks_by_type: by_type,
            total_failures,
        })
    }

    /// Run the scheduler background loop
    pub async fn run(self: Arc<Self>) -> Result<()> {
        tracing::info!("Scheduler started");
        
        loop {
            // Check for due tasks every 30 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            if let Ok(due_tasks) = self.get_due_tasks().await {
                for task in due_tasks {
                    if let Err(e) = self.execute_task(&task.id).await {
                        tracing::error!("Failed to execute task {}: {}", task.id, e);
                    }
                }
            }
        }
    }
}

/// Trait for task handlers
pub trait TaskHandler: Send + Sync {
    fn execute(&self) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;
}

impl<T: Fn() -> F + Send + Sync, F: std::future::Future<Output = Result<()>> + Send + 'static> TaskHandler for T {
    fn execute(&self) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> {
        Box::pin(self())
    }
}

/// Statistics about the scheduler
#[derive(Debug)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub tasks_by_status: std::collections::HashMap<TaskStatus, usize>,
    pub tasks_by_type: std::collections::HashMap<TaskType, usize>,
    pub total_failures: u32,
}
