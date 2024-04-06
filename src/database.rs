//  DATABASE.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:26:16
//  Last edited:
//    06 Apr 2024, 15:29:39
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an appropriate database abstraction for the DnD server.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::Path;

use async_sqlite::Pool;


/***** ERRORS *****/
/// Defines errors originating from the [`Database`].
#[derive(Debug)]
pub enum Error {
    /// It's an SQLite error.
    SQLite(SQLiteError),
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            SQLite {  }
        }
    }
}





/***** LIBRARY *****/
/// A database abstraction for the DnD server.
///
/// Currently, the only possible abstraction is one over an SQLite database, implemented with the [`async_sqlite`] crate.
#[derive(Debug)]
pub struct Database {
    pool: Pool,
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
    pub fn sqlite(path: impl AsRef<Path>) -> Result<Self, Error> {}
}
