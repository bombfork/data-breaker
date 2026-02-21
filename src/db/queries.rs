use rusqlite::params;

use super::Database;
use super::models::{Broker, DeletionRequest, PersonalRecord};

impl Database {
    // --- Brokers ---

    pub fn upsert_broker(&self, broker: &Broker) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO brokers (id, name, website, description, category, connector, registry_updated_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                website = excluded.website,
                description = excluded.description,
                category = excluded.category,
                connector = excluded.connector,
                registry_updated_at = excluded.registry_updated_at,
                updated_at = excluded.updated_at",
            params![
                broker.id,
                broker.name,
                broker.website,
                broker.description,
                broker.category,
                broker.connector,
                broker.registry_updated_at,
                broker.created_at,
                broker.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_broker(&self, id: &str) -> anyhow::Result<Option<Broker>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, website, description, category, connector, registry_updated_at, created_at, updated_at
             FROM brokers WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Broker {
                id: row.get(0)?,
                name: row.get(1)?,
                website: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                connector: row.get(5)?,
                registry_updated_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn list_brokers(&self, category: Option<&str>) -> anyhow::Result<Vec<Broker>> {
        let conn = self.conn.lock().unwrap();
        let mut brokers = Vec::new();

        if let Some(cat) = category {
            let mut stmt = conn.prepare(
                "SELECT id, name, website, description, category, connector, registry_updated_at, created_at, updated_at
                 FROM brokers WHERE category = ?1 ORDER BY name",
            )?;
            let rows = stmt.query_map(params![cat], |row| {
                Ok(Broker {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    website: row.get(2)?,
                    description: row.get(3)?,
                    category: row.get(4)?,
                    connector: row.get(5)?,
                    registry_updated_at: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?;
            for row in rows {
                brokers.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, name, website, description, category, connector, registry_updated_at, created_at, updated_at
                 FROM brokers ORDER BY name",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Broker {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    website: row.get(2)?,
                    description: row.get(3)?,
                    category: row.get(4)?,
                    connector: row.get(5)?,
                    registry_updated_at: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?;
            for row in rows {
                brokers.push(row?);
            }
        }

        Ok(brokers)
    }

    // --- Personal Records ---

    pub fn upsert_personal_record(&self, record: &PersonalRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO personal_records (id, broker_id, data_type, data_value, profile_url, raw_json, found_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(broker_id, data_type, data_value) DO UPDATE SET
                profile_url = excluded.profile_url,
                raw_json = excluded.raw_json,
                found_at = excluded.found_at",
            params![
                record.id,
                record.broker_id,
                record.data_type,
                record.data_value,
                record.profile_url,
                record.raw_json,
                record.found_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_personal_record(&self, id: &str) -> anyhow::Result<Option<PersonalRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, broker_id, data_type, data_value, profile_url, raw_json, found_at
             FROM personal_records WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(PersonalRecord {
                id: row.get(0)?,
                broker_id: row.get(1)?,
                data_type: row.get(2)?,
                data_value: row.get(3)?,
                profile_url: row.get(4)?,
                raw_json: row.get(5)?,
                found_at: row.get(6)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn list_personal_records(
        &self,
        broker_id: Option<&str>,
    ) -> anyhow::Result<Vec<PersonalRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut records = Vec::new();

        if let Some(bid) = broker_id {
            let mut stmt = conn.prepare(
                "SELECT id, broker_id, data_type, data_value, profile_url, raw_json, found_at
                 FROM personal_records WHERE broker_id = ?1 ORDER BY found_at DESC",
            )?;
            let rows = stmt.query_map(params![bid], |row| {
                Ok(PersonalRecord {
                    id: row.get(0)?,
                    broker_id: row.get(1)?,
                    data_type: row.get(2)?,
                    data_value: row.get(3)?,
                    profile_url: row.get(4)?,
                    raw_json: row.get(5)?,
                    found_at: row.get(6)?,
                })
            })?;
            for row in rows {
                records.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, broker_id, data_type, data_value, profile_url, raw_json, found_at
                 FROM personal_records ORDER BY found_at DESC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(PersonalRecord {
                    id: row.get(0)?,
                    broker_id: row.get(1)?,
                    data_type: row.get(2)?,
                    data_value: row.get(3)?,
                    profile_url: row.get(4)?,
                    raw_json: row.get(5)?,
                    found_at: row.get(6)?,
                })
            })?;
            for row in rows {
                records.push(row?);
            }
        }

        Ok(records)
    }

    // --- Deletion Requests ---

    pub fn insert_deletion_request(&self, req: &DeletionRequest) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO deletion_requests (id, broker_id, personal_record_id, status, submitted_at, completed_at, error_message, external_ref, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                req.id,
                req.broker_id,
                req.personal_record_id,
                req.status,
                req.submitted_at,
                req.completed_at,
                req.error_message,
                req.external_ref,
                req.created_at,
                req.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_deletion_request(&self, req: &DeletionRequest) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE deletion_requests SET status = ?2, completed_at = ?3, error_message = ?4, updated_at = ?5
             WHERE id = ?1",
            params![
                req.id,
                req.status,
                req.completed_at,
                req.error_message,
                req.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_deletion_requests(
        &self,
        broker_id: Option<&str>,
    ) -> anyhow::Result<Vec<DeletionRequest>> {
        let conn = self.conn.lock().unwrap();
        let mut requests = Vec::new();

        if let Some(bid) = broker_id {
            let mut stmt = conn.prepare(
                "SELECT id, broker_id, personal_record_id, status, submitted_at, completed_at, error_message, external_ref, created_at, updated_at
                 FROM deletion_requests WHERE broker_id = ?1 ORDER BY created_at DESC",
            )?;
            let rows = stmt.query_map(params![bid], Self::map_deletion_row)?;
            for row in rows {
                requests.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, broker_id, personal_record_id, status, submitted_at, completed_at, error_message, external_ref, created_at, updated_at
                 FROM deletion_requests ORDER BY created_at DESC",
            )?;
            let rows = stmt.query_map([], Self::map_deletion_row)?;
            for row in rows {
                requests.push(row?);
            }
        }

        Ok(requests)
    }

    fn map_deletion_row(row: &rusqlite::Row) -> rusqlite::Result<DeletionRequest> {
        Ok(DeletionRequest {
            id: row.get(0)?,
            broker_id: row.get(1)?,
            personal_record_id: row.get(2)?,
            status: row.get(3)?,
            submitted_at: row.get(4)?,
            completed_at: row.get(5)?,
            error_message: row.get(6)?,
            external_ref: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }

    // --- Registry Meta ---

    pub fn set_registry_meta(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO registry_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_registry_meta(&self, key: &str) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM registry_meta WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}
