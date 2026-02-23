pub mod broker;
pub mod delete;
pub mod registry;
pub mod report;
pub mod scan;
pub mod status;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "data-breaker",
    version,
    about = "Break what data brokers do — automate personal data removal"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage the broker registry
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    /// List and inspect known data brokers
    Broker {
        #[command(subcommand)]
        command: BrokerCommand,
    },
    /// Scan data brokers for your personal information
    Scan {
        /// First name to search for
        #[arg(long)]
        first_name: String,
        /// Last name to search for
        #[arg(long)]
        last_name: String,
        /// Email address to search for
        #[arg(long)]
        email: Option<String>,
        /// Phone number to search for
        #[arg(long)]
        phone: Option<String>,
        /// City for location-based searches
        #[arg(long)]
        city: Option<String>,
        /// State for location-based searches
        #[arg(long)]
        state: Option<String>,
        /// Your country (ISO 3166-1 alpha-2, e.g. US, GB, DE) — filters to relevant brokers
        #[arg(long)]
        country: Option<String>,
        /// Only scan specific brokers (comma-separated IDs)
        #[arg(long, value_delimiter = ',')]
        brokers: Vec<String>,
    },
    /// Request deletion of your personal data
    Delete {
        /// Delete all found records across all brokers
        #[arg(long, conflicts_with_all = ["broker", "record"])]
        all: bool,
        /// Delete records from a specific broker
        #[arg(long, conflicts_with = "record")]
        broker: Option<String>,
        /// Delete a specific record by ID
        #[arg(long)]
        record: Option<String>,
    },
    /// Check the status of deletion requests
    Status {
        /// Filter by broker ID
        #[arg(long)]
        broker: Option<String>,
        /// Filter by status (pending, submitted, in_progress, completed, failed, rejected)
        #[arg(long)]
        filter: Option<String>,
    },
    /// Generate a report of findings and deletion status
    Report {
        /// Output format
        #[arg(long, default_value = "terminal", value_parser = ["terminal", "json", "html"])]
        format: String,
        /// Output file path (stdout if not specified)
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum RegistryCommand {
    /// Fetch the latest broker registry from GitHub
    Update,
    /// Show registry metadata
    Info,
}

#[derive(Subcommand)]
pub enum BrokerCommand {
    /// List known data brokers
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Filter by broker home country (ISO 3166-1 alpha-2)
        #[arg(long)]
        country: Option<String>,
    },
    /// Show details about a specific broker
    Info {
        /// Broker ID (slug)
        id: String,
    },
}
