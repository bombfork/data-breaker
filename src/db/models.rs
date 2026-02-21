use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Broker {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub connector: Option<String>,
    pub registry_updated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalRecord {
    pub id: String,
    pub broker_id: String,
    pub data_type: String,
    pub data_value: String,
    pub profile_url: Option<String>,
    pub raw_json: Option<String>,
    pub found_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRequest {
    pub id: String,
    pub broker_id: String,
    pub personal_record_id: Option<String>,
    pub status: String,
    pub submitted_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub external_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
