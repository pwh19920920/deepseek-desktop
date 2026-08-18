use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to resolve dsh path: {0}")]
    DshPath(#[from] anyhow::Error),

    #[error("Failed to spawn sidecar: {0}")]
    Spawn(#[from] anyhow::Error),

    #[error("Sidecar exited unexpectedly: {0}")]
    SidecarExit(String),

    #[error("Port discovery failed: {0}")]
    PortDiscovery(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
