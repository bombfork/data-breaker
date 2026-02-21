use serde::Deserialize;

use crate::config::REGISTRY_URL;
use crate::db::models::Broker;

#[derive(Debug, Deserialize)]
struct RegistryBroker {
    id: String,
    name: String,
    website: Option<String>,
    description: Option<String>,
    category: Option<String>,
    connector: Option<String>,
}

/// Fetch the broker registry from the remote URL and return Broker models.
pub async fn fetch_registry() -> anyhow::Result<Vec<Broker>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(REGISTRY_URL)
        .header("User-Agent", "data-breaker")
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch registry: HTTP {}", resp.status());
    }

    let registry_brokers: Vec<RegistryBroker> = resp.json().await?;
    let now = chrono::Utc::now().to_rfc3339();

    let brokers = registry_brokers
        .into_iter()
        .map(|rb| Broker {
            id: rb.id,
            name: rb.name,
            website: rb.website,
            description: rb.description,
            category: rb.category,
            connector: rb.connector,
            registry_updated_at: Some(now.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
        })
        .collect();

    Ok(brokers)
}
