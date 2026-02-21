use std::collections::HashMap;
use std::sync::Arc;

use comfy_table::{Cell, Table};

use crate::broker::{BrokerConnector, PersonQuery};
use crate::db::Database;
use crate::db::models::{Broker, PersonalRecord};

pub async fn scan(
    db: &Database,
    connectors: &HashMap<String, Arc<dyn BrokerConnector>>,
    query: &PersonQuery,
    broker_filter: &[String],
) -> anyhow::Result<()> {
    let active_connectors: Vec<_> = if broker_filter.is_empty() {
        connectors.iter().collect()
    } else {
        connectors
            .iter()
            .filter(|(id, _)| broker_filter.contains(id))
            .collect()
    };

    if active_connectors.is_empty() {
        println!("No matching connectors found. Available connectors:");
        for id in connectors.keys() {
            println!("  - {id}");
        }
        return Ok(());
    }

    let mut total_found = 0usize;

    for (id, connector) in &active_connectors {
        if !connector.capabilities().can_scan {
            tracing::info!("Skipping {} (no scan capability)", id);
            continue;
        }

        // Ensure broker exists in DB (for FK constraint)
        if db.get_broker(id)?.is_none() {
            let now = chrono::Utc::now().to_rfc3339();
            db.upsert_broker(&Broker {
                id: id.to_string(),
                name: connector.name().to_string(),
                website: None,
                description: None,
                category: None,
                connector: Some(id.to_string()),
                registry_updated_at: None,
                created_at: now.clone(),
                updated_at: now,
            })?;
        }

        println!("Scanning {}...", connector.name());
        match connector.scan(query).await {
            Ok(records) => {
                if records.is_empty() {
                    println!("  No records found.");
                    continue;
                }

                for record in &records {
                    let personal_record = PersonalRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        broker_id: id.to_string(),
                        data_type: record.data_type.clone(),
                        data_value: record.data_value.clone(),
                        profile_url: record.profile_url.clone(),
                        raw_json: record
                            .metadata
                            .as_ref()
                            .map(|m| serde_json::to_string(m).unwrap_or_default()),
                        found_at: chrono::Utc::now().to_rfc3339(),
                    };
                    db.upsert_personal_record(&personal_record)?;
                }

                total_found += records.len();
                println!("  Found {} record(s).", records.len());
            }
            Err(e) => {
                tracing::error!("Error scanning {}: {}", id, e);
                println!("  Error: {e}");
            }
        }
    }

    // Show summary table
    let all_records = db.list_personal_records(None)?;
    if !all_records.is_empty() {
        println!("\n--- Scan Results ---");
        let mut table = Table::new();
        table.set_header(vec!["Broker", "Type", "Value", "Profile URL"]);

        for r in &all_records {
            table.add_row(vec![
                Cell::new(&r.broker_id),
                Cell::new(&r.data_type),
                Cell::new(&r.data_value),
                Cell::new(r.profile_url.as_deref().unwrap_or("-")),
            ]);
        }

        println!("{table}");
    }

    println!("\nTotal new records found this scan: {total_found}");
    Ok(())
}
