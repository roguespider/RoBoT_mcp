// src/bridge/acp.rs
// ACP (Agent Communication Protocol) for agent-to-agent communication
//
// NOTE: This module is a placeholder for future multi-agent communication.
// Currently unused but kept for future expansion.

#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// ACP message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub id: String,
    pub sender: AcpAgentId,
    pub receiver: AcpAgentId,
    pub message_type: AcpMessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub conversation_id: Option<String>,
    pub reply_to: Option<String>,
}

impl AcpMessage {
    /// Create a new ACP message
    pub fn new(
        sender: AcpAgentId,
        receiver: AcpAgentId,
        message_type: AcpMessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            receiver,
            message_type,
            payload,
            timestamp: Utc::now(),
            conversation_id: None,
            reply_to: None,
        }
    }

    /// Create a reply to this message
    pub fn reply(&self, payload: serde_json::Value) -> AcpMessage {
        let mut reply = Self::new(
            self.receiver.clone(),
            self.sender.clone(),
            self.message_type.reply_type(),
            payload,
        );
        reply.conversation_id = self.conversation_id.clone();
        reply.reply_to = Some(self.id.clone());
        reply
    }
}

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcpAgentId {
    pub agent_type: String,
    pub instance_id: String,
}

impl AcpAgentId {
    /// Create a new agent ID
    pub fn new(agent_type: &str, instance_id: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            instance_id: instance_id.to_string(),
        }
    }

    /// Create a new agent ID with a random instance
    pub fn with_random_instance(agent_type: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            instance_id: Uuid::new_v4().to_string()[..8].to_string(),
        }
    }

    /// Get the full agent URI
    pub fn uri(&self) -> String {
        format!("acp://{}/{}", self.agent_type, self.instance_id)
    }
}

impl std::fmt::Display for AcpAgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.agent_type, self.instance_id)
    }
}

/// ACP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpMessageType {
    /// Request: asking the receiver to perform an action
    Request,
    /// Response: responding to a request
    Response,
    /// Query: asking for information
    Query,
    /// Inform: informing the receiver of something
    Inform,
    /// Acknowledge: acknowledging receipt of a message
    Ack,
    /// Error: reporting an error
    Error,
    /// Subscribe: subscribe to updates
    Subscribe,
    /// Unsubscribe: unsubscribe from updates
    Unsubscribe,
    /// Publish: publish an event
    Publish,
}

impl AcpMessageType {
    /// Get the reply message type for this message type
    pub fn reply_type(&self) -> Self {
        match self {
            Self::Request => Self::Response,
            Self::Query => Self::Response,
            Self::Inform => Self::Ack,
            _ => Self::Inform,
        }
    }
}

/// ACP protocol errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpError {
    pub code: AcpErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AcpError {
    pub fn new(code: AcpErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// ACP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpErrorCode {
    MalformedMessage,
    UnknownReceiver,
    NotAuthorized,
    NotFound,
    InvalidPayload,
    Timeout,
    InternalError,
}

/// ACP channel for sending and receiving messages
pub trait AcpChannel: Send + Sync {
    /// Send a message through the channel
    fn send(&self, message: AcpMessage) -> Result<()>;

    /// Receive a message from the channel (non-blocking)
    fn try_recv(&self) -> Result<Option<AcpMessage>>;

    /// Receive a message from the channel (blocking)
    fn recv(&self) -> Result<AcpMessage>;
}

/// ACP agent trait
pub trait AcpAgent: Send + Sync {
    /// Get the agent's ID
    fn id(&self) -> &AcpAgentId;

    /// Handle an incoming ACP message
    fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>>;

    /// Get the agent's capabilities
    fn capabilities(&self) -> Vec<AcpCapability>;
}

/// Agent capability description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpCapability {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

/// ACP registry for agent discovery
pub struct AcpRegistry {
    agents: std::sync::RwLock<std::collections::HashMap<AcpAgentId, Arc<dyn AcpAgent>>>,
}

impl AcpRegistry {
    pub fn new() -> Self {
        Self {
            agents: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Register an agent
    pub fn register(&self, agent: Arc<dyn AcpAgent>) -> Result<()> {
        let id = agent.id().clone();
        let mut agents = self
            .agents
            .write()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        agents.insert(id, agent);
        Ok(())
    }

    /// Unregister an agent
    pub fn unregister(&self, id: &AcpAgentId) -> Result<Option<Arc<dyn AcpAgent>>> {
        let mut agents = self
            .agents
            .write()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(agents.remove(id))
    }

    /// Get an agent by ID
    pub fn get(&self, id: &AcpAgentId) -> Result<Option<Arc<dyn AcpAgent>>> {
        let agents = self
            .agents
            .read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(agents.get(id).cloned())
    }

    /// List all registered agent IDs
    pub fn list_agents(&self) -> Result<Vec<AcpAgentId>> {
        let agents = self
            .agents
            .read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(agents.keys().cloned().collect())
    }
}

impl Default for AcpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// ACP router for routing messages between agents
pub struct AcpRouter {
    registry: Arc<AcpRegistry>,
}

impl AcpRouter {
    pub fn new(registry: Arc<AcpRegistry>) -> Self {
        Self { registry }
    }

    /// Route a message to the appropriate agent
    pub fn route(&self, message: AcpMessage) -> Result<Option<AcpMessage>> {
        let agent = self.registry.get(&message.receiver)?;

        match agent {
            Some(agent) => agent.handle(message),
            None => Err(anyhow::anyhow!("Unknown receiver: {}", message.receiver)),
        }
    }

    /// Get the registry
    pub fn registry(&self) -> Arc<AcpRegistry> {
        Arc::clone(&self.registry)
    }
}
