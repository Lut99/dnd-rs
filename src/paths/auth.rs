//  AUTH.rs
//    by Lut99
//
//  Created:
//    09 Apr 2024, 12:18:07
//  Last edited:
//    09 Apr 2024, 12:49:44
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides handlers for logging users in.
//!   
//!   Logging out is simply done by the client discarding the login token.
//

use std::borrow::Cow;
use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use enum_debug::EnumDebug as _;
use error_trace::trace;
use hyper::StatusCode;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use crate::auth::{check_password, check_token, create_token};
use crate::database::UserInfo;
use crate::spec::Path;
use crate::state::ServerState;


/***** CONSTANTS *****/
/// The name of the login token cookie.
pub const LOGIN_TOKEN_NAME: &'static str = "login-token";





/***** SPEC *****/
/// The reqwest-compatible path on which the version endpoint can be found.
pub const PATH: Path = Path { method: hyper::Method::GET, path: "/v1/version" };


/// The request's body as given by the user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginRequest<'a> {
    /// The name of the user to login.
    pub name: Cow<'a, str>,
    /// The password proving the user is who we think they are.
    pub pass: Cow<'a, str>,
}





/***** LIBRARY *****/
/// Handles logging users in.
///
/// # Arguments
/// - `state`: The shared [`ServerState`] between paths.
/// - `client`: The address of the client we're working with.
/// - `jar`: A [`PrivateCookieJar`] that we use to store cookies in.
/// - `body`: A [`LoginRequest`] that contains the username/password to login with.
///
/// # Returns
/// `200 OK` with the login token as a new cookie.
///
/// `400 BAD REQUEST` if the given `body` was invalid.
///
/// `401 NOT AUTHORIZED` if the username was not found _or_ the password was invalid for that user.
///
/// # Errors
/// This function may error (with `500 INTERNAL SERVER ERROR`) if we fail to hash the given password or fail to contact the backend database.
#[cfg_attr(feature = "axum-debug", axum_macros::debug_handler)]
pub async fn login(
    State(state): State<ServerState>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    jar: PrivateCookieJar,
    Json(body): Json<LoginRequest<'static>>,
) -> (StatusCode, PrivateCookieJar, String) {
    info!("Handling {} {} from '{}'", PATH.method, PATH.path, client);

    // Check if the user is already logged-in with a valid token
    if let Some(token) = jar.get(LOGIN_TOKEN_NAME) {
        // Ensure it's still valid!
        debug!("Client presents us with login token {token:?}, checking validity");
        match check_token(&state.db, token.value()) {
            // It is, nothing to do
            Ok(Ok(token)) => {
                debug!("Client '{}' login token is valid for user {} (role: {}), nothing to do", client, token.id, token.role.variant());
                return (StatusCode::OK, jar, String::new());
            },
            // It's invalid. Continue to insert.
            Ok(Err(err)) => {
                debug!("{}", trace!(("Client '{client}' login token is not valid, logging user in"), err));
            },
            // An error occurred
            Err(err) => {
                error!("{}", trace!(("Failed to check token {:?} validity", token.value()), err));
                return (StatusCode::INTERNAL_SERVER_ERROR, jar, String::new());
            },
        }
    }

    // Attempt to find this user in the database
    debug!("Retrieving user '{}' from database", body.name);
    let user: UserInfo = match state.db.get_user_by_name(body.name.as_ref()) {
        Ok(Some(user)) => user,
        Ok(None) => {
            debug!("User '{}' not found, returning 401 UNAUTHORIZED", body.name);
            return (StatusCode::UNAUTHORIZED, jar, String::new());
        },
        Err(err) => {
            error!("{}", trace!(("Failed to get user info for user '{}' from database", body.name), err));
            return (StatusCode::INTERNAL_SERVER_ERROR, jar, format!("Failed to get user info for user '{}' from database", body.name));
        },
    };

    // Check the passwords
    debug!("Doing password gate-check...");
    if !check_password(&body.pass, &user.pass) {
        debug!("User '{}' password incorrect, returning 401 UNAUTHORIZED", body.name);
        return (StatusCode::UNAUTHORIZED, jar, String::new());
    }

    // Alrighty that's it, generate a new token and return that
    debug!("User '{}' password correct, generating token", body.name);
    match create_token(user.id, user.role) {
        Ok(token) => (StatusCode::OK, jar.add(Cookie::new(LOGIN_TOKEN_NAME, token)), String::new()),
        Err(err) => {
            error!("{}", trace!(("Failed to get generate login token for user '{}'", body.name), err));
            return (StatusCode::INTERNAL_SERVER_ERROR, jar, format!("Failed to get generate login token for user '{}'", body.name));
        },
    }
}
