pub mod dummy;
pub mod registry;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonQuery {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundRecord {
    pub data_type: String,
    pub data_value: String,
    pub profile_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionSubmission {
    pub external_ref: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionStatusCheck {
    pub status: String,
    pub completed_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConnectorCapabilities {
    pub can_scan: bool,
    pub can_delete: bool,
    pub can_check_status: bool,
}

#[async_trait]
pub trait BrokerConnector: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> ConnectorCapabilities;
    async fn scan(&self, query: &PersonQuery) -> anyhow::Result<Vec<FoundRecord>>;
    async fn request_deletion(
        &self,
        query: &PersonQuery,
        records: &[FoundRecord],
    ) -> anyhow::Result<DeletionSubmission>;
    async fn check_deletion_status(
        &self,
        external_ref: &str,
    ) -> anyhow::Result<DeletionStatusCheck>;
}

/// Build the map of all compiled-in connectors.
/// Contributors: add your connector here.
pub fn build_connector_registry() -> HashMap<String, Arc<dyn BrokerConnector>> {
    let mut map: HashMap<String, Arc<dyn BrokerConnector>> = HashMap::new();

    let dummy = Arc::new(dummy::DummyBroker);
    map.insert(dummy.id().to_string(), dummy);

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_connector_registry() {
        let reg = build_connector_registry();
        assert!(reg.contains_key("dummy-broker"));
        let dummy = reg.get("dummy-broker").unwrap();
        assert_eq!(dummy.name(), "Dummy Broker");
        assert!(dummy.capabilities().can_scan);
        assert!(dummy.capabilities().can_delete);
        assert!(dummy.capabilities().can_check_status);
    }

    #[tokio::test]
    async fn test_dummy_scan() {
        let dummy = dummy::DummyBroker;
        let query = PersonQuery {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: Some("john@example.com".into()),
            phone: None,
            city: None,
            state: None,
        };
        let results = dummy.scan(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_dummy_deletion() {
        let dummy = dummy::DummyBroker;
        let query = PersonQuery {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: None,
            phone: None,
            city: None,
            state: None,
        };
        let records = dummy.scan(&query).await.unwrap();
        let submission = dummy.request_deletion(&query, &records).await.unwrap();
        assert!(!submission.external_ref.is_empty());

        let status = dummy
            .check_deletion_status(&submission.external_ref)
            .await
            .unwrap();
        assert_eq!(status.status, "in_progress");
    }
}
