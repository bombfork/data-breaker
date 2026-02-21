use comfy_table::{Cell, Table};

use crate::db::Database;

pub fn list_brokers(db: &Database, category: Option<&str>) -> anyhow::Result<()> {
    let brokers = db.list_brokers(category)?;

    if brokers.is_empty() {
        println!(
            "No brokers found. Run `data-breaker registry update` to fetch the broker registry."
        );
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Category", "Website"]);

    for b in &brokers {
        table.add_row(vec![
            Cell::new(&b.id),
            Cell::new(&b.name),
            Cell::new(b.category.as_deref().unwrap_or("-")),
            Cell::new(b.website.as_deref().unwrap_or("-")),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub fn broker_info(db: &Database, id: &str) -> anyhow::Result<()> {
    let broker = db.get_broker(id)?;
    match broker {
        Some(b) => {
            println!("ID:          {}", b.id);
            println!("Name:        {}", b.name);
            if let Some(w) = &b.website {
                println!("Website:     {w}");
            }
            if let Some(d) = &b.description {
                println!("Description: {d}");
            }
            if let Some(c) = &b.category {
                println!("Category:    {c}");
            }
            if let Some(conn) = &b.connector {
                println!("Connector:   {conn}");
            }
            println!("Updated:     {}", b.updated_at);
        }
        None => {
            anyhow::bail!("Broker '{}' not found", id);
        }
    }
    Ok(())
}
