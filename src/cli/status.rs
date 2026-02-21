use std::collections::HashMap;
use std::sync::Arc;

use comfy_table::{Cell, Table};

use crate::broker::BrokerConnector;
use crate::db::Database;

pub async fn status(
    db: &Database,
    connectors: &HashMap<String, Arc<dyn BrokerConnector>>,
    broker_filter: Option<&str>,
    status_filter: Option<&str>,
) -> anyhow::Result<()> {
    let mut requests = db.list_deletion_requests(broker_filter)?;

    if let Some(filter) = status_filter {
        requests.retain(|r| r.status == filter);
    }

    if requests.is_empty() {
        println!("No deletion requests found.");
        return Ok(());
    }

    // Check for status updates on in-flight requests
    for req in &mut requests {
        if matches!(req.status.as_str(), "submitted" | "in_progress")
            && let Some(ext_ref) = &req.external_ref
            && let Some(connector) = connectors.get(&req.broker_id)
            && connector.capabilities().can_check_status
        {
            match connector.check_deletion_status(ext_ref).await {
                Ok(check) => {
                    if check.status != req.status {
                        req.status = check.status.clone();
                        req.completed_at = check.completed_at.clone();
                        req.error_message = check.message.clone();
                        req.updated_at = chrono::Utc::now().to_rfc3339();
                        db.update_deletion_request(req)?;
                        tracing::info!("Updated status for {} -> {}", req.id, check.status);
                    }
                }
                Err(e) => {
                    tracing::warn!("Could not check status for {}: {}", req.id, e);
                }
            }
        }
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Broker", "Status", "Submitted", "External Ref"]);

    for req in &requests {
        table.add_row(vec![
            Cell::new(&req.id[..8]),
            Cell::new(&req.broker_id),
            Cell::new(&req.status),
            Cell::new(req.submitted_at.as_deref().unwrap_or("-")),
            Cell::new(req.external_ref.as_deref().unwrap_or("-")),
        ]);
    }

    println!("{table}");
    Ok(())
}
