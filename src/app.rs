// src/app.rs

use std::sync::Arc;

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::{Scheduler, TaskSchedule, TaskType};
use crate::experience::scorer::ExperienceScorer;
use crate::bridge::mcp::McpContext;
use crate::bridge::rmcp::run_stdio_server;
use crate::tools;


/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Persistent database layer.
    _database: Arc<SqliteDatabase>,

    /// Event bus for pub/sub.
    #[allow(dead_code)]
    bus: Arc<ExperienceBus>,

    /// Experience system coordinator.
    #[allow(dead_code)]
    coordinator: Arc<ExperienceCoordinator>,

    /// Reflection engine for learning from experiences.
    #[allow(dead_code)]
    reflection_engine: Arc<ReflectionEngine>,

    /// Evolution engine for behavior management.
    #[allow(dead_code)]
    evolution_engine: Arc<EvolutionEngine>,

    /// Background task scheduler.
    scheduler: Arc<Scheduler>,

    /// Metrics collector.
    #[allow(dead_code)]
    metrics: Arc<MetricsCollector>,

    /// MCP context shared with bridge.
    mcp_context: Arc<McpContext>,
}


impl App {

    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = Arc::new(SqliteDatabase::initialize()?);

        // Create core systems
        let bus = Arc::new(ExperienceBus::new());
        let scorer = ExperienceScorer::new();
        let coordinator = Arc::new(ExperienceCoordinator::new(scorer));
        
        // Create learning engines
        let reflection_engine = Arc::new(ReflectionEngine::new());
        let evolution_engine = Arc::new(EvolutionEngine::new());
        
        // Create scheduler with background tasks
        let scheduler = Self::setup_scheduler(database.clone()).await?;
        
        // Create metrics collector
        let metrics = Arc::new(MetricsCollector::new());

        // Create MCP context
        let mcp_context = Arc::new(McpContext::new(
            database.clone(),
            bus.clone(),
            coordinator.clone(),
            reflection_engine.clone(),
            evolution_engine.clone(),
            scheduler.clone(),
            metrics.clone(),
        ));

        // Register MCP tools
        tools::register_tools(&mcp_context);

        tracing::info!("RoBoT initialized successfully");

        Ok(Self {
            _database: database,
            bus,
            coordinator,
            reflection_engine,
            evolution_engine,
            scheduler,
            metrics,
            mcp_context,
        })
    }

    /// Setup background task scheduler with default tasks.
    async fn setup_scheduler(database: Arc<SqliteDatabase>) -> Result<Arc<Scheduler>> {
        let scheduler = Arc::new(Scheduler::new(database));
        
        // Schedule periodic reflection (every 30 minutes)
        scheduler.create_task(
            "periodic_reflection",
            TaskType::Reflection,
            TaskSchedule::Interval { seconds: 1800 },
        ).await?;

        // Schedule hypothesis evaluation (every hour)
        scheduler.create_task(
            "hypothesis_evaluation",
            TaskType::HypothesisEvaluation,
            TaskSchedule::Interval { seconds: 3600 },
        ).await?;

        // Schedule metrics collection (every 5 minutes)
        scheduler.create_task(
            "metrics_collection",
            TaskType::MetricsCollection,
            TaskSchedule::Interval { seconds: 300 },
        ).await?;

        // Schedule evolution maintenance (every day at midnight)
        scheduler.create_task(
            "evolution_maintenance",
            TaskType::EvolutionMaintenance,
            TaskSchedule::Daily { hour: 0, minute: 0 },
        ).await?;

        Ok(scheduler)
    }

    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
        // Start background scheduler worker
        let scheduler = self.scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });

        // Run the MCP server with stdio transport
        run_stdio_server(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            self.mcp_context.clone(),
        ).await
    }
}
