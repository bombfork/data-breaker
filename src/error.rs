use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Broker not found: {0}")]
    BrokerNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("No records to delete")]
    NoRecordsToDelete,

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
