use super::Report;

pub fn render(report: &Report) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}
