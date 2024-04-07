//  DATABASE.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:26:16
//  Last edited:
//    07 Apr 2024, 14:46:44
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an appropriate database abstraction for the DnD server.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};

use async_sqlite::rusqlite::Connection;
use async_sqlite::{Pool, PoolBuilder};
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::fs as tfs;


/***** ERRORS *****/
/// Defines errors originating from the [`Database`].
#[derive(Debug)]
pub enum Error {
    /// Failed to parse the root's credentials file as TOML.
    RootCredsParse { path: PathBuf, err: toml::de::Error },
    /// Failed to read the root's credentials file.
    RootCredsRead { path: PathBuf, err: std::io::Error },

    /// It's an SQLite error.
    SQLite(SQLiteError),
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            RootCredsParse { path, .. } => write!(f, "Failed to parse root credentials file '{}' as valid TOML", path.display()),
            RootCredsRead { path, .. } => write!(f, "Failed to read root credentials file '{}'", path.display()),

            SQLite(err) => write!(f, "{err}"),
        }
    }
}
impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            RootCredsParse { err, .. } => Some(err),
            RootCredsRead { err, .. } => Some(err),

            SQLite(err) => Some(err),
        }
    }
}



/// Defines errors originating from the [`Database`] when it uses the SQLite backend.
#[derive(Debug)]
pub enum SQLiteError {
    /// Failed to create a new connection [`Pool`].
    PoolCreate { path: PathBuf, err: async_sqlite::Error },
}
impl Display for SQLiteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use SQLiteError::*;
        match self {
            PoolCreate { path, .. } => write!(f, "Failed to create SQLite connection pool to '{}'", path.display()),
        }
    }
}
impl error::Error for SQLiteError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use SQLiteError::*;
        match self {
            PoolCreate { err, .. } => Some(err),
        }
    }
}





/***** AUXILLARY *****/
/// The layout of the root credentials file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RootCreds {
    /// The name of the root user.
    name: String,
    /// The password for the root user.
    pass: String,
}





/***** LIBRARY *****/
/// A database abstraction for the DnD server.
///
/// Currently, the only possible abstraction is one over an SQLite database, implemented with the [`async_sqlite`] crate.
pub enum Database {
    SQLite {
        /// The path to the database file we use for debugging.
        path: PathBuf,
        /// The SQLite connection [`Pool`] which we use for connections.
        pool: Pool,
    },
}
impl Database {
    /// Constructor for the Database that uses the SQLite backend.
    ///
    /// # Arguments
    /// - `path`: The path on which the SQLite database to connect with lives.
    ///
    /// # Returns
    /// A new Database to use.
    ///
    /// # Errors
    /// This function errors if we failed to build a connection pool to that database.
    #[inline]
    pub fn sqlite(path: impl Into<PathBuf>) -> Result<Self, Error> {
        // Build a connection pool to the path
        let path: PathBuf = path.into();
        match PoolBuilder::new().path(&path).open_blocking() {
            Ok(pool) => Ok(Self::SQLite { path, pool }),
            Err(err) => Err(Error::SQLite(SQLiteError::PoolCreate { path, err })),
        }
    }

    /// Initializes the backend database with the required tables and such.
    ///
    /// # Arguments
    /// - `root_path`: The path to the [`RootConfig`] file that describes how to generate the root user.
    ///
    /// # Errors
    /// This function can error if we failed to write to the backend database.
    pub async fn init(&self, root_path: impl AsRef<Path>) -> Result<(), Error> {
        // Load the root config file
        let root_path: &Path = root_path.as_ref();
        debug!("Loading root credentials file '{}'...", root_path.display());
        let root_creds: String = match tfs::read_to_string(root_path).await {
            Ok(text) => text,
            Err(err) => return Err(Error::RootCredsRead { path: root_path.into(), err }),
        };
        let root_creds: RootCreds = match toml::from_str(&root_creds) {
            Ok(creds) => creds,
            Err(err) => return Err(Error::RootCredsParse { path: root_path.into(), err }),
        };


        // Now initialize based on the backend
        match self {
            Self::SQLite { path, pool } => {
                debug!("Initializing database file '{}'...", path.display());

                debug!("Connecting to database file '{}'...", path.display());
                let conn: Connection = match pool.conn(async move {}) {};

                // OK run
                Ok(())
            },
        }
    }
}
