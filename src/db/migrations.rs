use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    // Migration 1: Initial schema
    "CREATE TABLE IF NOT EXISTS brokers (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        website TEXT,
        description TEXT,
        category TEXT,
        connector TEXT,
        registry_updated_at TEXT,
        created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );

    CREATE TABLE IF NOT EXISTS personal_records (
        id TEXT PRIMARY KEY,
        broker_id TEXT NOT NULL REFERENCES brokers(id),
        data_type TEXT NOT NULL,
        data_value TEXT NOT NULL,
        profile_url TEXT,
        raw_json TEXT,
        found_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        UNIQUE(broker_id, data_type, data_value)
    );

    CREATE TABLE IF NOT EXISTS deletion_requests (
        id TEXT PRIMARY KEY,
        broker_id TEXT NOT NULL REFERENCES brokers(id),
        personal_record_id TEXT REFERENCES personal_records(id),
        status TEXT NOT NULL DEFAULT 'pending',
        submitted_at TEXT,
        completed_at TEXT,
        error_message TEXT,
        external_ref TEXT,
        created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );

    CREATE TABLE IF NOT EXISTS registry_meta (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );",
];

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY)")?;

    let current_version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM _migrations",
        [],
        |row| row.get(0),
    )?;

    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i64;
        if version > current_version {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO _migrations (version) VALUES (?1)", [version])?;
            tracing::info!("Applied migration {version}");
        }
    }

    Ok(())
}
