//  MAIN.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:12:56
//  Last edited:
//    06 Apr 2024, 15:23:47
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint to the DnD server binary.
//

use std::path::PathBuf;

use clap::Parser;
use humanlog::{DebugMode, HumanLogger};
use log::info;


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
}





/***** LIBRARY *****/
fn main() {
    // Parse CLI args
    let args = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(if args.verbose { DebugMode::Full } else { DebugMode::Debug }).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (logging disabled for this session)");
    }
    info!("{} v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
}
