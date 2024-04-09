//  MAIN.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:12:56
//  Last edited:
//    09 Apr 2024, 12:13:45
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint to the DnD server binary.
//

use std::fs;
use std::future::IntoFuture as _;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr as _;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use clap::Parser;
use dnd_server::database::Database;
use dnd_server::paths;
use dnd_server::state::ServerState;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};
use semver::Version;
use tokio::net::TcpListener;
use tokio::runtime::{Builder, Runtime};
use tokio::signal::unix::{signal, Signal, SignalKind};


/***** ARGUMENTS *****/
/// Defines arguments for the binary.
#[derive(Parser)]
struct Arguments {
    /// If given, enables more verbose logging.
    #[clap(short, long, global = true)]
    verbose: bool,

    /// The address on which to host the server.
    #[clap(short, long, global = true, default_value = "0.0.0.0:4200")]
    address:   SocketAddr,
    /// The path to the persistent data file.
    #[clap(short, long, global = true, default_value = "/data/data.db")]
    data_path: PathBuf,
    /// The path to the root's credentials file. This is only used if the database needs to be initialized to generate the root user.
    #[clap(short, long, global = true, default_value = "/config/root.toml")]
    root_path: PathBuf,
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



    /* Database */
    // Touch the database file alive if it doesn't exist
    let needs_init: bool = if !args.data_path.exists() {
        // Doesn't exist; touch the file and return it needs initing
        debug!("Database file '{}' does not exist", args.data_path.display());
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
    let db: Database = Database::sqlite(&args.data_path);

    // If it needs initialization, do so
    if needs_init {
        debug!("Initializing database...");
        if let Err(err) = db.init(&args.root_path) {
            error!("{}", trace!(("Failed to initialize database file '{}'", args.data_path.display()), err));
            std::process::exit(1);
        }
    }



    /* PATH BUILDING */
    // Create a runtime state out of that
    let state: Arc<ServerState> = ServerState::arced(env!("CARGO_BIN_NAME"), Version::from_str(env!("CARGO_PKG_VERSION")).unwrap(), db);

    // Build the health path
    debug!("Building axum paths...");
    let version: Router = Router::new().route("/version", get(paths::version::handle)).with_state(state);
    let routes: Router = Router::new().nest("/v1", version);



    /* EXECUTION */
    // Build a tokio runtime to enter async mode
    debug!("Building tokio runtime...");
    let runtime: Runtime = match Builder::new_multi_thread().enable_io().enable_time().build() {
        Ok(runtime) => runtime,
        Err(err) => {
            error!("{}", trace!(("Failed to create tokio runtime"), err));
            std::process::exit(1);
        },
    };
    std::process::exit(runtime.block_on(async move {
        // Bind a listener on the specified address for the server
        debug!("Binding server listener to '{}'...", args.address);
        let listener: TcpListener = match TcpListener::bind(args.address).await {
            Ok(listener) => listener,
            Err(err) => {
                error!("{}", trace!(("Failed to bind to '{}'", args.address), err));
                return 1;
            },
        };

        // Build a listener for SIGTERM
        debug!("Registering SIGTERM handler...");
        let mut sigterm: Signal = match signal(SignalKind::terminate()) {
            Ok(handler) => handler,
            Err(err) => {
                error!("{}", trace!(("Failed to create SIGTERM handler"), err));
                return 1;
            },
        };

        // Run the server in a loop while alternating with listening for signals
        info!("Initialization complete, entering game loop");
        tokio::select! {
            // Let the server handle the stuff
            res = axum::serve(listener, routes.into_make_service_with_connect_info::<SocketAddr>()).into_future() => match res {
                Ok(_) => 0,
                Err(err) => {
                    error!("{}", trace!(("Failed to run axum server"), err));
                    1
                }
            },

            // Wait for SIGTERM to be super Docker-friendly
            _ = sigterm.recv() => 0,
        }
    }));
}
