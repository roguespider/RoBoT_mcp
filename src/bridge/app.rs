// src/bridge/app.rs
// Root application container per Architecture §03

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::mcp::McpContext;
use crate::bridge::mcp::McpClient;
use crate::bridge::rmcp::run_stdio_server;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::event_handler::EventHandler;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::{Scheduler, TaskSchedule, TaskType};
use crate::experience::scorer::ExperienceScorer;
use crate::knowledge::KnowledgeStore;
use crate::learning::{WorkingMemory, LineageTracker};
use crate::memory::{MemoryRetrieval, WorkingMemory as MemWorkingMemory, PermanentMemory};
use crate::planner::{Planner, PolicyEngine};
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

    /// Background task scheduler.
    scheduler: Arc<Scheduler>,

    /// MCP context shared with bridge - owns all subsystems.
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
        let coordinator = Arc::new(ExperienceCoordinator::new(scorer, bus.clone()));

        // Start event handler to process events from the bus
        let event_handler = EventHandler::new(bus.clone());
        event_handler.start();
        tracing::info!("Event handler started");
        
        // Create learning engines
        let reflection_engine = Arc::new(ReflectionEngine::new());
        let evolution_engine = Arc::new(EvolutionEngine::new());
        
        // Create working memory, lineage tracker, and knowledge store
        let _working_memory = Arc::new(WorkingMemory::new(1000));
        let _lineage_tracker = Arc::new(LineageTracker::new());
        let knowledge_store = Arc::new(KnowledgeStore::new(10000));
        
        // Create memory system - Working and Permanent Memory (Architecture §6.3)
        let working_memory_core = Arc::new(MemWorkingMemory::new(1000));
        let permanent_memory = Arc::new(PermanentMemory::new(10000));
        let memory_retrieval = Arc::new(MemoryRetrieval::new(
            working_memory_core.clone(),
            permanent_memory.clone(),
        ));
        tracing::info!("Memory system initialized (Working: 1000, Permanent: 10000)");
        
        // Create scheduler with background tasks
        let scheduler = Self::setup_scheduler(database.clone()).await?;

        // Register task handlers
        Self::register_task_handlers(scheduler.clone()).await;

        // Start scheduler background loop
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler_clone.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });
        tracing::info!("Scheduler background loop started");
        
        // Create metrics collector
        let metrics = Arc::new(MetricsCollector::new());

        // Create planning system (Architecture §4.03.5, §10)
        let planner = Arc::new(Planner::new(metrics.clone()));
        let policy_engine = Arc::new(PolicyEngine::new());
        
        // Load default policy rules
        policy_engine.load_defaults().await;
        tracing::info!("Policy engine loaded with default rules");

        // Create MCP context with all systems
        let mcp_context = Arc::new(McpContext::new(
            database.clone(),
            bus.clone(),
            coordinator.clone(),
            reflection_engine.clone(),
            evolution_engine.clone(),
            scheduler.clone(),
            metrics.clone(),
            knowledge_store.clone(),
            planner.clone(),
            policy_engine.clone(),
            working_memory_core.clone(),
            permanent_memory.clone(),
            memory_retrieval.clone(),
        ));

        // Register MCP tools
        tools::register_tools(&mcp_context);

        // Create MCP client for external connections and initialize globally
        crate::tools::agent::init_mcp_client(Arc::new(McpClient::new()));

        tracing::info!("RoBoT initialized successfully");

        Ok(Self {
            _database: database,
            bus,
            coordinator,
            scheduler,
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

    /// Register task handlers for the scheduler
    async fn register_task_handlers(scheduler: Arc<Scheduler>) {
        use crate::experience::scheduler::TaskType;

        // Reflection task handler
        scheduler.register_handler(TaskType::Reflection, Box::new(|| {
            Box::pin(async move {
                tracing::info!("Executing scheduled reflection task");
                Ok(())
            })
        })).await;

        // Hypothesis evaluation handler
        scheduler.register_handler(TaskType::HypothesisEvaluation, Box::new(|| {
            Box::pin(async move {
                tracing::info!("Executing scheduled hypothesis evaluation");
                Ok(())
            })
        })).await;

        // Metrics collection handler
        scheduler.register_handler(TaskType::MetricsCollection, Box::new(|| {
            Box::pin(async move {
                tracing::debug!("Executing scheduled metrics collection");
                Ok(())
            })
        })).await;

        // Evolution maintenance handler
        scheduler.register_handler(TaskType::EvolutionMaintenance, Box::new(|| {
            Box::pin(async move {
                tracing::info!("Executing scheduled evolution maintenance");
                Ok(())
            })
        })).await;

        // Exploration analysis handler
        scheduler.register_handler(TaskType::ExplorationAnalysis, Box::new(|| {
            Box::pin(async move {
                tracing::debug!("Executing scheduled exploration analysis");
                Ok(())
            })
        })).await;

        // Cleanup handler
        scheduler.register_handler(TaskType::Cleanup, Box::new(|| {
            Box::pin(async move {
                tracing::info!("Executing scheduled cleanup");
                Ok(())
            })
        })).await;

        // Reputation decay handler
        scheduler.register_handler(TaskType::ReputationDecay, Box::new(|| {
            Box::pin(async move {
                tracing::debug!("Executing scheduled reputation decay");
                Ok(())
            })
        })).await;

        tracing::info!("Registered {} task handlers", 7);
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
