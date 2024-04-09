//  VERSION.rs
//    by Lut99
//
//  Created:
//    08 Apr 2024, 17:36:28
//  Last edited:
//    09 Apr 2024, 12:49:53
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a simple, no-nonsense `version` endpoint that can be used to
//!   extract some information.
//

use std::borrow::Cow;
use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::response::Json;
use hyper::StatusCode;
use log::info;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::spec::Path;
use crate::state::ServerState;


/***** SPEC *****/
/// The reqwest-compatible path on which the version endpoint can be found.
pub const PATH: Path = Path { method: hyper::Method::GET, path: "/v1/version" };


/// The response returned by the version endpoint.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VersionResponse<'a> {
    /// The name of the server executable.
    pub name:    Cow<'a, str>,
    /// The semantic version of the server.
    pub version: Version,
}





/***** LIBRARY *****/
/// Handles `GET /v1/version` to return the current server information to a client.
///
/// # Arguments
/// - `state`: The shared [`ServerState`] between paths.
/// - `client`: The address of the client we're working with.
///
/// # Returns
/// `200 OK` with a [`VersionResponse`] in the body.
#[cfg_attr(feature = "axum-debug", axum_macros::debug_handler)]
pub async fn handle(State(state): State<ServerState>, ConnectInfo(client): ConnectInfo<SocketAddr>) -> (StatusCode, Json<VersionResponse<'static>>) {
    info!("Handling {} {} from '{}'", PATH.method, PATH.path, client);
    (StatusCode::OK, Json::from(VersionResponse { name: Cow::Borrowed(state.name), version: state.version.clone() }))
}
