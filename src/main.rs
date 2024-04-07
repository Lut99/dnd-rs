//  MAIN.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:12:56
//  Last edited:
//    07 Apr 2024, 14:44:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint to the DnD server binary.
//

use std::fs::{self, File};
use std::path::PathBuf;

use clap::Parser;
use dnd_server::database::Database;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};


/***** ARGUMENTS *****/
/// Defines arguments for the binary.
#[derive(Parser)]
struct Arguments {
    /// If given, enables more verbose logging.
    #[clap(short, long, global = true)]
    verbose: bool,

    /// The path to the persistent data file.
    #[clap(short, long, global = true, default_value = "/data/data.db")]
    data_path: PathBuf,
    /// The path to the root's credentials file. This is only used if the database needs to be initialized to generate the root user.
    #[clap(short, long, global = true, default_value = "/config/root.toml")]
    root_path: PathBuf,
}





/***** LIBRARY *****/
#[tokio::main]
async fn main() {
    // Parse CLI args
    let args = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(if args.verbose { DebugMode::Full } else { DebugMode::Debug }).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (logging disabled for this session)");
    }
    info!("{} v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));



    /* Database */
    // Touch the database file alive if it doesn't exist
    let needs_init: bool = if !args.data_path.exists() {
        // Doesn't exist; touch the file and return it needs initing
        debug!("Database file '{}' does not exist, creating file...", args.data_path.display());
        if let Err(err) = File::create(&args.data_path) {
            error!("{}", trace!(("Failed to touch database file '{}'", args.data_path.display()), err));
            std::process::exit(1);
        }
        true
    } else {
        // Already exists, no init please
        debug!("Database file '{}' already exists", args.data_path.display());

        // ...unless its empty!
        match fs::metadata(&args.data_path) {
            Ok(md) => {
                if md.len() == 0 {
                    debug!("Database file '{}' is uninitialized", args.data_path.display());
                    true
                } else {
                    false
                }
            },
            Err(err) => {
                error!("{}", trace!(("Failed to get database file '{}' metadata", args.data_path.display()), err));
                std::process::exit(1);
            },
        }
    };

    // Open a connection to the database
    let db: Database = match Database::sqlite(&args.data_path) {
        Ok(db) => db,
        Err(err) => {
            error!("{}", trace!(("Failed to setup database"), err));
            std::process::exit(1);
        },
    };

    // If it needs initialization, do so
    if needs_init {
        debug!("Initializing database...");
        if let Err(err) = db.init(&args.root_path).await {
            error!("{}", trace!(("Failed to initialize database file '{}'", args.data_path.display()), err));
            std::process::exit(1);
        }
    }
}
