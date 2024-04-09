//  STATE.rs
//    by Lut99
//
//  Created:
//    08 Apr 2024, 11:55:37
//  Last edited:
//    09 Apr 2024, 12:49:21
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the shared [`ServerState`] between all path handlers.
//

use std::ops::Deref;
use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use semver::Version;

use crate::database::Database;


/***** LIBRARY *****/
/// Defines the shared state between all path handlers.
///
/// This is the out-facing struct, already wrapping the [`ServerStateInternal`] in an [`Arc`].
#[derive(Clone, Debug)]
pub struct ServerState(Arc<InternalServerState>);
impl ServerState {
    /// Constructor for the ServerState.
    ///
    /// # Arguments
    /// - `name`: Some name for the server executable that can be shared with clients upon request.
    /// - `version`: Some version for the server executable that can be shared with clients upon request.
    /// - `db`: Some already initialized [`Database`] connection to use to store persistent state.
    ///
    /// # Returns
    /// A new ServerState.
    #[inline]
    pub fn new(name: &'static str, version: Version, db: Database) -> Self { Self(Arc::new(InternalServerState::new(name, version, db))) }
}
impl Deref for ServerState {
    type Target = InternalServerState;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl FromRef<ServerState> for Key {
    fn from_ref(input: &ServerState) -> Self { input.0.key.clone() }
}



/// Defines the shared state between all path handlers.
///
/// This is the internal struct, which is yet to be wrapped in an [`Arc`].
#[derive(Debug)]
pub struct InternalServerState {
    /// The name of the server executable.
    pub name:    &'static str,
    /// The (parsed!) version of the server executable.
    pub version: Version,

    /// The database that we use for the data-wise state.
    pub db: Database,

    /// Some key that we generate every time the server starts.
    pub key: Key,
}
impl InternalServerState {
    /// Constructor for the InternalServerState.
    ///
    /// # Arguments
    /// - `name`: Some name for the server executable that can be shared with clients upon request.
    /// - `version`: Some version for the server executable that can be shared with clients upon request.
    /// - `db`: Some already initialized [`Database`] connection to use to store persistent state.
    ///
    /// # Returns
    /// A new InternalServerState.
    #[inline]
    pub fn new(name: &'static str, version: Version, db: Database) -> Self { Self { name, version, db, key: Key::generate() } }
}
