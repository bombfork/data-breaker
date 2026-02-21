use crate::db::Database;
use crate::report::{Report, ReportFormat};

pub fn generate_report(db: &Database, format: &str, output: Option<&str>) -> anyhow::Result<()> {
    let report = Report::build(db)?;

    let fmt = match format {
        "json" => ReportFormat::Json,
        "html" => ReportFormat::Html,
        _ => ReportFormat::Terminal,
    };

    let rendered = report.render(fmt)?;

    match output {
        Some(path) => {
            std::fs::write(path, &rendered)?;
            println!("Report written to {path}");
        }
        None => {
            println!("{rendered}");
        }
    }

    Ok(())
}
