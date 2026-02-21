pub mod html;
pub mod json;
pub mod terminal;

use serde::Serialize;

use crate::db::Database;
use crate::db::models::{Broker, DeletionRequest, PersonalRecord};

pub enum ReportFormat {
    Terminal,
    Json,
    Html,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub generated_at: String,
    pub brokers: Vec<Broker>,
    pub records: Vec<PersonalRecord>,
    pub deletion_requests: Vec<DeletionRequest>,
    pub summary: ReportSummary,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub total_brokers: usize,
    pub total_records: usize,
    pub total_deletions: usize,
    pub deletions_pending: usize,
    pub deletions_submitted: usize,
    pub deletions_completed: usize,
    pub deletions_failed: usize,
}

impl Report {
    pub fn build(db: &Database) -> anyhow::Result<Self> {
        let brokers = db.list_brokers(None)?;
        let records = db.list_personal_records(None)?;
        let deletion_requests = db.list_deletion_requests(None)?;

        let summary = ReportSummary {
            total_brokers: brokers.len(),
            total_records: records.len(),
            total_deletions: deletion_requests.len(),
            deletions_pending: deletion_requests
                .iter()
                .filter(|r| r.status == "pending")
                .count(),
            deletions_submitted: deletion_requests
                .iter()
                .filter(|r| r.status == "submitted")
                .count(),
            deletions_completed: deletion_requests
                .iter()
                .filter(|r| r.status == "completed")
                .count(),
            deletions_failed: deletion_requests
                .iter()
                .filter(|r| r.status == "failed" || r.status == "rejected")
                .count(),
        };

        Ok(Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            brokers,
            records,
            deletion_requests,
            summary,
        })
    }

    pub fn render(&self, format: ReportFormat) -> anyhow::Result<String> {
        match format {
            ReportFormat::Terminal => terminal::render(self),
            ReportFormat::Json => json::render(self),
            ReportFormat::Html => html::render(self),
        }
    }
}
