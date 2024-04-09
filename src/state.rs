//  STATE.rs
//    by Lut99
//
//  Created:
//    08 Apr 2024, 11:55:37
//  Last edited:
//    09 Apr 2024, 11:58:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the shared [`ServerState`] between all path handlers.
//

use std::sync::Arc;

use semver::Version;

use crate::database::Database;


/***** LIBRARY *****/
/// Defines the shared state between all path handlers.
pub struct ServerState {
    /// The name of the server executable.
    pub name:    &'static str,
    /// The (parsed!) version of the server executable.
    pub version: Version,

    /// The database that we use for the data-wise state.
    pub db: Database,
}
impl ServerState {
    /// Constructor for the ServerState that returns it wrapped in an [`Arc`].
    ///
    /// # Arguments
    /// - `name`: Some name for the server executable that can be shared with clients upon request.
    /// - `version`: Some version for the server executable that can be shared with clients upon request.
    /// - `db`: Some already initialized [`Database`] connection to use to store persistent state.
    ///
    /// # Returns
    /// A new ServerState wrapped in an [`Arc`].
    #[inline]
    pub fn arced(name: &'static str, version: Version, db: Database) -> Arc<Self> { Arc::new(Self { name, version, db }) }
}
