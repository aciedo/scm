use clap::{Parser, Subcommand};
use human_panic::setup_panic;

mod env;
mod migration;

/// An ergonomic ScyllaDB migration manager
#[derive(Debug, Parser)]
#[command(name = "scm")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Creates a migration file
    #[command(arg_required_else_help = true)]
    Create {
        /// A title of the migration.
        name: String,
    },
    /// Applies migrations
    #[command()]
    Apply {
        /// The migration to apply
        migration: Option<String>,
        #[arg(short, long)]
        /// The environment to use
        env: Option<String>,
    },
    /// Lists migrations
    #[command()]
    List,
    /// Environment management
    #[command(arg_required_else_help = true)]
    Env {
        #[command(subcommand)]
        command: EnvironmentCommands,
    },
}

#[derive(Debug, Subcommand)]
enum EnvironmentCommands {
    /// Lists environments
    #[command()]
    List,
    /// Creates a new environment
    #[command()]
    Create {
        #[arg(default_value = "dev")]
        /// The name of the environment
        name: String,
        /// The address of the ScyllaDB node
        /// (default: localhost)
        #[arg(default_value = "localhost")]
        host: String,
    },
    /// Deletes an environment
    #[command(arg_required_else_help = true)]
    Delete {
        /// The name of the environment
        name: String,
    },
}

#[tokio::main]
async fn main() {
    setup_panic!();
    let args = Cli::parse();

    match args.command {
        Commands::Create { name } => migration::create(name),
        Commands::Apply { migration, env } => migration::apply(migration, env).await,
        Commands::List => migration::list_migrations(),
        Commands::Env { command } => match command {
            EnvironmentCommands::List => env::list(),
            EnvironmentCommands::Create { name, host } => env::create(name, host),
            EnvironmentCommands::Delete { name } => env::delete(name),
        },
    }
}
