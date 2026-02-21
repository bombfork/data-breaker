use super::Report;

pub fn render(report: &Report) -> anyhow::Result<String> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>Data Breaker Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("  body { font-family: system-ui, sans-serif; max-width: 960px; margin: 2rem auto; padding: 0 1rem; color: #1a1a1a; }\n");
    html.push_str("  h1 { border-bottom: 2px solid #333; padding-bottom: 0.5rem; }\n");
    html.push_str("  table { border-collapse: collapse; width: 100%; margin: 1rem 0; }\n");
    html.push_str("  th, td { border: 1px solid #ddd; padding: 0.5rem; text-align: left; }\n");
    html.push_str("  th { background: #f5f5f5; font-weight: 600; }\n");
    html.push_str("  tr:nth-child(even) { background: #fafafa; }\n");
    html.push_str("  .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; margin: 1rem 0; }\n");
    html.push_str("  .stat { background: #f5f5f5; padding: 1rem; border-radius: 4px; }\n");
    html.push_str("  .stat .value { font-size: 1.5rem; font-weight: 700; }\n");
    html.push_str("  .stat .label { color: #666; font-size: 0.875rem; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    html.push_str(&format!(
        "<h1>Data Breaker Report</h1>\n<p>Generated: {}</p>\n",
        report.generated_at
    ));

    // Summary cards
    html.push_str("<div class=\"summary\">\n");
    write_stat(&mut html, "Brokers Tracked", report.summary.total_brokers);
    write_stat(&mut html, "Records Found", report.summary.total_records);
    write_stat(
        &mut html,
        "Deletions Submitted",
        report.summary.deletions_submitted,
    );
    write_stat(
        &mut html,
        "Deletions Completed",
        report.summary.deletions_completed,
    );
    write_stat(
        &mut html,
        "Deletions Failed",
        report.summary.deletions_failed,
    );
    html.push_str("</div>\n");

    // Records table
    if !report.records.is_empty() {
        html.push_str("<h2>Personal Records Found</h2>\n");
        html.push_str("<table>\n<thead><tr><th>Broker</th><th>Type</th><th>Value</th><th>Found At</th></tr></thead>\n<tbody>\n");
        for r in &report.records {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&r.broker_id),
                escape_html(&r.data_type),
                escape_html(&r.data_value),
                escape_html(&r.found_at),
            ));
        }
        html.push_str("</tbody></table>\n");
    }

    // Deletion requests table
    if !report.deletion_requests.is_empty() {
        html.push_str("<h2>Deletion Requests</h2>\n");
        html.push_str("<table>\n<thead><tr><th>ID</th><th>Broker</th><th>Status</th><th>Submitted</th></tr></thead>\n<tbody>\n");
        for req in &report.deletion_requests {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&req.id[..8]),
                escape_html(&req.broker_id),
                escape_html(&req.status),
                escape_html(req.submitted_at.as_deref().unwrap_or("-")),
            ));
        }
        html.push_str("</tbody></table>\n");
    }

    html.push_str("</body>\n</html>\n");

    Ok(html)
}

fn write_stat(html: &mut String, label: &str, value: usize) {
    html.push_str(&format!(
        "<div class=\"stat\"><div class=\"value\">{value}</div><div class=\"label\">{label}</div></div>\n"
    ));
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
