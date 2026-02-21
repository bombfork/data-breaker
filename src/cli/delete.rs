use std::collections::HashMap;
use std::sync::Arc;

use crate::broker::{BrokerConnector, PersonQuery};
use crate::db::Database;
use crate::db::models::DeletionRequest;

pub async fn delete(
    db: &Database,
    connectors: &HashMap<String, Arc<dyn BrokerConnector>>,
    query: &PersonQuery,
    all: bool,
    broker_id: Option<&str>,
    record_id: Option<&str>,
) -> anyhow::Result<()> {
    let records = if let Some(rid) = record_id {
        let r = db
            .get_personal_record(rid)?
            .ok_or_else(|| anyhow::anyhow!("Record '{}' not found", rid))?;
        vec![r]
    } else if let Some(bid) = broker_id {
        let records = db.list_personal_records(Some(bid))?;
        if records.is_empty() {
            anyhow::bail!("No records found for broker '{}'", bid);
        }
        records
    } else if all {
        let records = db.list_personal_records(None)?;
        if records.is_empty() {
            anyhow::bail!("No records found. Run `data-breaker scan` first.");
        }
        records
    } else {
        anyhow::bail!("Specify --all, --broker <id>, or --record <id>");
    };

    // Group records by broker
    let mut by_broker: HashMap<String, Vec<_>> = HashMap::new();
    for r in &records {
        by_broker
            .entry(r.broker_id.clone())
            .or_default()
            .push(r.clone());
    }

    let mut submitted = 0usize;
    let mut failed = 0usize;

    for (bid, broker_records) in &by_broker {
        let connector = match connectors.get(bid.as_str()) {
            Some(c) => c,
            None => {
                println!("No connector for broker '{}', skipping.", bid);
                failed += broker_records.len();
                continue;
            }
        };

        if !connector.capabilities().can_delete {
            println!(
                "Connector '{}' does not support deletion, skipping.",
                connector.name()
            );
            failed += broker_records.len();
            continue;
        }

        let found_records: Vec<_> = broker_records
            .iter()
            .map(|r| crate::broker::FoundRecord {
                data_type: r.data_type.clone(),
                data_value: r.data_value.clone(),
                profile_url: r.profile_url.clone(),
                metadata: None,
            })
            .collect();

        println!("Requesting deletion from {}...", connector.name());
        match connector.request_deletion(query, &found_records).await {
            Ok(submission) => {
                let now = chrono::Utc::now().to_rfc3339();
                for r in broker_records {
                    let deletion = DeletionRequest {
                        id: uuid::Uuid::new_v4().to_string(),
                        broker_id: bid.clone(),
                        personal_record_id: Some(r.id.clone()),
                        status: "submitted".to_string(),
                        submitted_at: Some(now.clone()),
                        completed_at: None,
                        error_message: None,
                        external_ref: Some(submission.external_ref.clone()),
                        created_at: now.clone(),
                        updated_at: now.clone(),
                    };
                    db.insert_deletion_request(&deletion)?;
                }
                submitted += broker_records.len();
                println!("  Submitted (ref: {})", submission.external_ref);
            }
            Err(e) => {
                tracing::error!("Error deleting from {}: {}", bid, e);
                println!("  Error: {e}");
                failed += broker_records.len();
            }
        }
    }

    println!("\nDeletion requests: {submitted} submitted, {failed} failed");
    Ok(())
}
