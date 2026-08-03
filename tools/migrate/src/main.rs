//! Postgres → Turso row-level snapshot migration tool.
//!
//! Commands:
//!   export --source <sqlite-path> --out snapshot.json
//!   import --target <sqlite-path> --in snapshot.json
//!   verify --target <sqlite-path> --in snapshot.json

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use staple_migrate::{export, import, load_snapshot, verify};

#[derive(Parser)]
#[command(name = "staple-migrate", about = "Postgres → Turso snapshot migration")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Exports rows from a source database into a snapshot file.
    Export {
        /// Source database path (local SQLite/libsql; Postgres URL support planned).
        #[arg(long)]
        source: String,
        /// Output snapshot file.
        #[arg(long)]
        out: PathBuf,
    },
    /// Imports a snapshot into a Turso database (runs migrations first).
    Import {
        /// Target database path.
        #[arg(long)]
        target: String,
        /// Snapshot file.
        #[arg(long)]
        r#in: PathBuf,
    },
    /// Verifies row counts between a snapshot and a database.
    Verify {
        /// Target database path.
        #[arg(long)]
        target: String,
        /// Snapshot file.
        #[arg(long)]
        r#in: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Export { source, out } => {
            let snapshot = export(&source).await?;
            let bytes = serde_json::to_vec_pretty(&snapshot)?;
            std::fs::write(&out, bytes).with_context(|| format!("writing {}", out.display()))?;
            println!("exported {} tables to {}", snapshot.len(), out.display());
        }
        Command::Import { target, r#in } => {
            let snapshot = load_snapshot(&r#in)?;
            let counts = import(&target, &snapshot).await?;
            println!("imported: {counts:?}");
        }
        Command::Verify { target, r#in } => {
            let snapshot = load_snapshot(&r#in)?;
            verify(&target, &snapshot).await?;
        }
    }
    Ok(())
}
