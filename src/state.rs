//  STATE.rs
//    by Lut99
//
//  Created:
//    08 Apr 2024, 11:55:37
//  Last edited:
//    08 Apr 2024, 11:57:31
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the shared [`ServerState`] between all path handlers.
//

use std::sync::Arc;

use parking_lot::RwLock;

use crate::database::Database;


/***** LIBRARY *****/
/// Defines the shared state between all path handlers.
pub struct ServerState {
    /// The database that we use for the data-wise state.
    pub db: RwLock<Database>,
}
impl ServerState {
    /// Constructor for the ServerState that returns it wrapped in an [`Arc`].
    ///
    /// # Arguments
    /// - `db`: Some already initialized [`Database`] connection to use to store persistent state.
    ///
    /// # Returns
    /// A new ServerState wrapped in an [`Arc`].
    #[inline]
    pub fn arced(db: Database) -> Arc<Self> { Arc::new(Self { db: RwLock::new(db) }) }
}
