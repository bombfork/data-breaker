use crate::db::Database;

pub async fn update_registry(db: &Database) -> anyhow::Result<()> {
    println!("Fetching broker registry...");
    let brokers = crate::broker::registry::fetch_registry().await?;
    let count = brokers.len();

    for broker in brokers {
        db.upsert_broker(&broker)?;
    }

    let now = chrono::Utc::now().to_rfc3339();
    db.set_registry_meta("last_fetched_at", &now)?;

    println!("Registry updated: {count} broker(s) synced.");
    Ok(())
}

pub fn registry_info(db: &Database) -> anyhow::Result<()> {
    let last_fetched = db.get_registry_meta("last_fetched_at")?;
    let broker_count = db.list_brokers(None)?.len();

    match last_fetched {
        Some(ts) => println!("Last updated:  {ts}"),
        None => println!("Last updated:  never (run `data-breaker registry update`)"),
    }
    println!("Brokers known: {broker_count}");
    Ok(())
}
