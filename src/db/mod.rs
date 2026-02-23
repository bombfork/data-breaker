pub mod migrations;
pub mod models;
pub mod queries;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Broker, DeletionRequest, PersonalRecord};

    fn test_db() -> Database {
        Database::open_in_memory().expect("Failed to create test database")
    }

    #[test]
    fn test_migrations_run() {
        let db = test_db();
        let conn = db.conn.lock().unwrap();
        // Verify tables exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert!(count >= 1);
    }

    #[test]
    fn test_broker_crud() {
        let db = test_db();
        let now = chrono::Utc::now().to_rfc3339();
        let broker = Broker {
            id: "test-broker".into(),
            name: "Test Broker".into(),
            website: Some("https://example.com".into()),
            description: Some("A test broker".into()),
            category: Some("people-search".into()),
            connector: None,
            country: Some("US".into()),
            data_countries: Some("US,GB".into()),
            registry_updated_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        db.upsert_broker(&broker).unwrap();

        let fetched = db.get_broker("test-broker").unwrap().unwrap();
        assert_eq!(fetched.name, "Test Broker");
        assert_eq!(fetched.country.as_deref(), Some("US"));
        assert_eq!(fetched.data_countries.as_deref(), Some("US,GB"));

        let all = db.list_brokers(None, None, None).unwrap();
        assert_eq!(all.len(), 1);

        let filtered = db.list_brokers(Some("people-search"), None, None).unwrap();
        assert_eq!(filtered.len(), 1);

        let empty = db.list_brokers(Some("nonexistent"), None, None).unwrap();
        assert!(empty.is_empty());

        // Filter by country
        let by_country = db.list_brokers(None, Some("US"), None).unwrap();
        assert_eq!(by_country.len(), 1);

        let no_country = db.list_brokers(None, Some("DE"), None).unwrap();
        assert!(no_country.is_empty());

        // Filter by data_country
        let by_data = db.list_brokers(None, None, Some("GB")).unwrap();
        assert_eq!(by_data.len(), 1);

        let no_data = db.list_brokers(None, None, Some("FR")).unwrap();
        assert!(no_data.is_empty());
    }

    #[test]
    fn test_personal_record_crud() {
        let db = test_db();
        let now = chrono::Utc::now().to_rfc3339();

        // Need a broker first (FK)
        let broker = Broker {
            id: "test-broker".into(),
            name: "Test".into(),
            website: None,
            description: None,
            category: None,
            connector: None,
            country: None,
            data_countries: None,
            registry_updated_at: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.upsert_broker(&broker).unwrap();

        let record = PersonalRecord {
            id: "rec-1".into(),
            broker_id: "test-broker".into(),
            data_type: "name".into(),
            data_value: "John Doe".into(),
            profile_url: Some("https://example.com/john".into()),
            raw_json: None,
            found_at: now,
        };
        db.upsert_personal_record(&record).unwrap();

        let fetched = db.get_personal_record("rec-1").unwrap().unwrap();
        assert_eq!(fetched.data_value, "John Doe");

        let all = db.list_personal_records(None).unwrap();
        assert_eq!(all.len(), 1);

        let by_broker = db.list_personal_records(Some("test-broker")).unwrap();
        assert_eq!(by_broker.len(), 1);
    }

    #[test]
    fn test_deletion_request_crud() {
        let db = test_db();
        let now = chrono::Utc::now().to_rfc3339();

        let broker = Broker {
            id: "test-broker".into(),
            name: "Test".into(),
            website: None,
            description: None,
            category: None,
            connector: None,
            country: None,
            data_countries: None,
            registry_updated_at: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.upsert_broker(&broker).unwrap();

        let req = DeletionRequest {
            id: "del-1".into(),
            broker_id: "test-broker".into(),
            personal_record_id: None,
            status: "submitted".into(),
            submitted_at: Some(now.clone()),
            completed_at: None,
            error_message: None,
            external_ref: Some("ref-123".into()),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.insert_deletion_request(&req).unwrap();

        let all = db.list_deletion_requests(None).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].status, "submitted");
    }

    #[test]
    fn test_registry_meta() {
        let db = test_db();
        assert!(db.get_registry_meta("last_fetched_at").unwrap().is_none());

        db.set_registry_meta("last_fetched_at", "2024-01-01T00:00:00Z")
            .unwrap();
        let val = db.get_registry_meta("last_fetched_at").unwrap().unwrap();
        assert_eq!(val, "2024-01-01T00:00:00Z");

        // Update
        db.set_registry_meta("last_fetched_at", "2024-06-01T00:00:00Z")
            .unwrap();
        let val = db.get_registry_meta("last_fetched_at").unwrap().unwrap();
        assert_eq!(val, "2024-06-01T00:00:00Z");
    }
}
