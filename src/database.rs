//  DATABASE.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:26:16
//  Last edited:
//    08 Apr 2024, 12:30:00
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an appropriate database abstraction for the DnD server.
//

use std::fmt::{Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};
use std::{error, fs};

use chrono::{DateTime, Utc};
use log::{debug, trace};
use rusqlite::{Connection, OptionalExtension as _, Transaction};
use serde::{Deserialize, Serialize};

use crate::auth::{hash_password, Role};


/***** HELPER MACROS *****/
/// Does an execute without parameters.
macro_rules! execute {
    ($path:ident, $trans:ident, $query:literal) => {{
        let query: &'static str = $query;
        match $trans.execute(query, []) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::SQLite(SQLiteError::QueryExecute { path: $path.clone(), query: query.into(), err })),
        }
    }};
}

/// Does an execute with parameters.
macro_rules! prepare {
    ($path:ident, $trans:ident, $query:literal, $($param:expr),+) => {{
        let query: &'static str = $query;
        match $trans.execute(query, [$($param),+]) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::SQLite(SQLiteError::QueryExecute { path: $path.clone(), query: query.into(), err })),
        }
    }};
}





/***** ERRORS *****/
/// Defines errors originating from the [`Database`].
#[derive(Debug)]
pub enum Error {
    /// Failed to hash the given password.
    HashPassword { err: crate::auth::PasswordError },
    /// Failed to parse the root's file as TOML.
    RootFileParse { path: PathBuf, err: toml::de::Error },
    /// Failed to read the root's file.
    RootFileRead { path: PathBuf, err: std::io::Error },

    /// It's an SQLite error.
    SQLite(SQLiteError),
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            HashPassword { .. } => write!(f, "Failed to hash root password"),
            RootFileParse { path, .. } => write!(f, "Failed to parse root file '{}' as valid TOML", path.display()),
            RootFileRead { path, .. } => write!(f, "Failed to read root file '{}'", path.display()),

            SQLite(err) => write!(f, "{err}"),
        }
    }
}
impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            HashPassword { err } => Some(err),
            RootFileParse { err, .. } => Some(err),
            RootFileRead { err, .. } => Some(err),

            SQLite(err) => Some(err),
        }
    }
}



/// Defines errors originating from the [`Database`] when it uses the SQLite backend.
#[derive(Debug)]
pub enum SQLiteError {
    /// Failed to create a new [`Connection`].
    ConnCreate { path: PathBuf, err: rusqlite::Error },
    /// Failed to execute a given query.
    QueryExecute { path: PathBuf, query: String, err: rusqlite::Error },
    /// Failed to commit a [`Transaction`].
    TransactionCommit { path: PathBuf, err: rusqlite::Error },
    /// Failed to create a new [`Transaction`].
    TransactionCreate { path: PathBuf, err: rusqlite::Error },
}
impl Display for SQLiteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use SQLiteError::*;
        match self {
            ConnCreate { path, .. } => write!(f, "Failed to create SQLite connection to '{}'", path.display()),
            QueryExecute { path, query, .. } => write!(f, "Failed to execute query {query:?} at database '{}'", path.display()),
            TransactionCommit { path, .. } => write!(f, "Failed to commit transaction to database '{}'", path.display()),
            TransactionCreate { path, .. } => write!(f, "Failed to create transaction for database '{}'", path.display()),
        }
    }
}
impl error::Error for SQLiteError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use SQLiteError::*;
        match self {
            ConnCreate { err, .. } => Some(err),
            QueryExecute { err, .. } => Some(err),
            TransactionCommit { err, .. } => Some(err),
            TransactionCreate { err, .. } => Some(err),
        }
    }
}





/***** AUXILLARY *****/
/// The layout of the root file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RootFile {
    /// The root-section.
    pub root: Root,
}

/// The layout of the `[root]`-section in the root file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Root {
    /// The credentials of the root file.
    #[serde(alias = "credentials")]
    pub creds: RootCreds,
}

/// The layout of the `[root.creds]`-section in the root file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RootCreds {
    /// The name of the root user.
    name: String,
    /// The password for the root user.
    pass: String,
}



/// Describes everything we store about a user.
#[derive(Clone, Debug)]
pub struct UserInfo {
    /// The identifier of the user.
    pub id:    u64,
    /// The name of the user.
    pub name:  String,
    /// The password of the user, hashed.
    pub pass:  String,
    /// The role of the user.
    pub role:  Role,
    /// The time the user was added.
    pub added: DateTime<Utc>,
}





/***** LIBRARY *****/
/// A database abstraction for the DnD server.
///
/// Currently, the only possible abstraction is one over an SQLite database, implemented with the [`async_sqlite`] crate.
pub enum Database {
    SQLite {
        /// The path to the database file we use for debugging.
        path: PathBuf,
        /// The SQLite [`Connection`] which we use to talk to the database.
        conn: Connection,
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
        // Build a connection to the path
        let path: PathBuf = path.into();
        debug!("Initializing Database with SQLite backend to database file '{}'...", path.display());
        match Connection::open(&path) {
            Ok(conn) => Ok(Self::SQLite { path, conn }),
            Err(err) => Err(Error::SQLite(SQLiteError::ConnCreate { path, err })),
        }
    }

    /// Initializes the backend database with the required tables and such.
    ///
    /// # Arguments
    /// - `root_path`: The path to the [`RootConfig`] file that describes how to generate the root user.
    ///
    /// # Errors
    /// This function can error if we failed to write to the backend database.
    pub fn init(&mut self, root_path: impl AsRef<Path>) -> Result<(), Error> {
        // Load the root config file
        let root_path: &Path = root_path.as_ref();
        debug!("Loading root credentials file '{}'...", root_path.display());
        let root_file: String = match fs::read_to_string(root_path) {
            Ok(text) => text,
            Err(err) => return Err(Error::RootFileRead { path: root_path.into(), err }),
        };
        let root_file: RootFile = match toml::from_str(&root_file) {
            Ok(creds) => creds,
            Err(err) => return Err(Error::RootFileParse { path: root_path.into(), err }),
        };


        // Now initialize based on the backend
        match self {
            Self::SQLite { path, conn } => {
                debug!("Initializing database file '{}'...", path.display());

                // Open a transaction
                let trans: Transaction = match conn.transaction() {
                    Ok(trans) => trans,
                    Err(err) => return Err(Error::SQLite(SQLiteError::TransactionCreate { path: path.clone(), err })),
                };


                {
                    // Create the users database
                    trace!("Creating table 'users'...");
                    execute!(
                        path,
                        trans,
                        "CREATE TABLE users (id BIGINT UNSIGNED, name VARCHAR(32), password VARVAR(97), role TINYINT UNSIGNED, added TIMESTAMP)"
                    )?;

                    // Inject the root user
                    trace!("Injecting root user '{}'...", root_file.root.creds.name);

                    // Hash their password first
                    let hpass: String = match hash_password(&root_file.root.creds.pass) {
                        Ok(hash) => hash,
                        Err(err) => return Err(Error::HashPassword { err }),
                    };

                    // Run the query
                    prepare!(
                        path,
                        trans,
                        "INSERT INTO users (id, name, password, role, added) VALUES (0, ?, ?, 10, CURRENT_TIMESTAMP)",
                        &root_file.root.creds.name,
                        &hpass
                    )?;
                }


                // OK, commit and done!
                match trans.commit() {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::SQLite(SQLiteError::TransactionCommit { path: path.clone(), err })),
                }
            },
        }
    }

    /// Retrieves a [`UserInfo`] describing the properties of a user.
    ///
    /// # Arguments
    /// - `id`: The identifier of the user to retrieve the info for.
    ///
    /// # Returns
    /// A [`UserInfo`] describing it all, or else [`None`] if we didn't found such a user.
    ///
    /// # Errors
    /// This function may error if we failed to communicate with the database.
    pub fn get_user_by_id(&self, id: u64) -> Result<Option<UserInfo>, Error> {
        debug!("Retrieving user info by ID for user {id}...");
        match self {
            Self::SQLite { path, conn } => {
                let query: &'static str = "SELECT * FROM users WHERE id=?";
                match conn
                    .query_row(query, [id], |row| {
                        Ok(UserInfo {
                            id:    row.get("id")?,
                            name:  row.get("name")?,
                            pass:  row.get("password")?,
                            role:  row.get::<&'static str, u8>("role")?.try_into().expect("Got invalid role in database"),
                            added: row.get("added")?,
                        })
                    })
                    .optional()
                {
                    Ok(info) => Ok(info),
                    Err(err) => Err(Error::SQLite(SQLiteError::QueryExecute { path: path.clone(), query: query.into(), err })),
                }
            },
        }
    }
}
