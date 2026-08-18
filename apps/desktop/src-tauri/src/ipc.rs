use serde::{Deserialize, Serialize};

/// Message types for frontend ↔ Rust communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DshMessage {
    /// Request to start the sidecar
    Start,
    /// Request to stop the sidecar
    Stop,
    /// Request status
    Status,
    /// Sidecar started with URL
    Started(String),
    /// Sidecar stopped
    Stopped,
    /// Error occurred
    Error(String),
}
