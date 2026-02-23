mod broker;
mod cli;
mod config;
mod db;
mod error;
mod report;

use clap::Parser;
use cli::{BrokerCommand, Cli, Command, RegistryCommand};

use crate::broker::PersonQuery;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up tracing
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .with_target(false)
        .init();

    // Open database
    let db_path = config::db_path()?;
    let db = db::Database::open(&db_path)?;

    // Build connector registry
    let connectors = broker::build_connector_registry();

    match cli.command {
        Command::Registry { command } => match command {
            RegistryCommand::Update => cli::registry::update_registry(&db).await?,
            RegistryCommand::Info => cli::registry::registry_info(&db)?,
        },
        Command::Broker { command } => match command {
            BrokerCommand::List { category, country } => {
                cli::broker::list_brokers(&db, category.as_deref(), country.as_deref())?
            }
            BrokerCommand::Info { id } => cli::broker::broker_info(&db, &id)?,
        },
        Command::Scan {
            first_name,
            last_name,
            email,
            phone,
            city,
            state,
            country,
            brokers,
        } => {
            let query = PersonQuery {
                first_name,
                last_name,
                email,
                phone,
                city,
                state,
                country,
            };
            cli::scan::scan(&db, &connectors, &query, &brokers).await?;
        }
        Command::Delete {
            all,
            broker: broker_id,
            record,
        } => {
            // Build a minimal query for deletion — connectors may need it
            let query = PersonQuery {
                first_name: String::new(),
                last_name: String::new(),
                email: None,
                phone: None,
                city: None,
                state: None,
                country: None,
            };
            cli::delete::delete(
                &db,
                &connectors,
                &query,
                all,
                broker_id.as_deref(),
                record.as_deref(),
            )
            .await?;
        }
        Command::Status { broker, filter } => {
            cli::status::status(&db, &connectors, broker.as_deref(), filter.as_deref()).await?;
        }
        Command::Report { format, output } => {
            cli::report::generate_report(&db, &format, output.as_deref())?;
        }
    }

    Ok(())
}
