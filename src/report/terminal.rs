use comfy_table::{Cell, Table};

use super::Report;

pub fn render(report: &Report) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str(&format!(
        "=== Data Breaker Report ({}) ===\n\n",
        report.generated_at
    ));

    // Summary
    output.push_str("--- Summary ---\n");
    output.push_str(&format!(
        "Brokers tracked:      {}\n",
        report.summary.total_brokers
    ));
    output.push_str(&format!(
        "Records found:        {}\n",
        report.summary.total_records
    ));
    output.push_str(&format!(
        "Deletion requests:    {}\n",
        report.summary.total_deletions
    ));
    output.push_str(&format!(
        "  Pending:            {}\n",
        report.summary.deletions_pending
    ));
    output.push_str(&format!(
        "  Submitted:          {}\n",
        report.summary.deletions_submitted
    ));
    output.push_str(&format!(
        "  Completed:          {}\n",
        report.summary.deletions_completed
    ));
    output.push_str(&format!(
        "  Failed/Rejected:    {}\n",
        report.summary.deletions_failed
    ));

    // Records table
    if !report.records.is_empty() {
        output.push_str("\n--- Personal Records Found ---\n");
        let mut table = Table::new();
        table.set_header(vec!["Broker", "Type", "Value", "Found At"]);
        for r in &report.records {
            table.add_row(vec![
                Cell::new(&r.broker_id),
                Cell::new(&r.data_type),
                Cell::new(&r.data_value),
                Cell::new(&r.found_at),
            ]);
        }
        output.push_str(&table.to_string());
        output.push('\n');
    }

    // Deletion requests table
    if !report.deletion_requests.is_empty() {
        output.push_str("\n--- Deletion Requests ---\n");
        let mut table = Table::new();
        table.set_header(vec!["ID", "Broker", "Status", "Submitted", "External Ref"]);
        for req in &report.deletion_requests {
            table.add_row(vec![
                Cell::new(&req.id[..8]),
                Cell::new(&req.broker_id),
                Cell::new(&req.status),
                Cell::new(req.submitted_at.as_deref().unwrap_or("-")),
                Cell::new(req.external_ref.as_deref().unwrap_or("-")),
            ]);
        }
        output.push_str(&table.to_string());
        output.push('\n');
    }

    Ok(output)
}
