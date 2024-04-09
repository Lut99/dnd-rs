//  AUTH.rs
//    by Lut99
//
//  Created:
//    09 Apr 2024, 12:52:49
//  Last edited:
//    09 Apr 2024, 13:06:03
//  Auto updated?
//    Yes
//
//  Description:
//!   Handles checking the login token in every request and resolving that
//!   to a [`UserInfo`] or a `401 NOT AUTHORIZED`.
//

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use error_trace::trace;
use hyper::StatusCode;
use log::{debug, error, info};

use crate::auth::{check_token, LOGIN_TOKEN_NAME};
use crate::database::UserInfo;
use crate::state::ServerState;


/***** LIBRARY *****/
/// Handles checking the login token in every request and resolving that to a [`UserInfo`] or a `401 NOT AUTHORIZED`.
///
/// This middleware injects an extension that allows handlers downstream to query the [`UserInfo`] for this user.
///
/// # Arguments
/// - `state`: The [`ServerState`] that has the common state between paths (for us, this means the backend database).
/// - `client`: Some [`SocketAddr`] of the client that connected.
/// - `jar`: A [`PrivateCookieJar`] that hopefully contains the login token.
/// - `request`: A [`Request`] to pass to some...
/// - `next`: A [`Next`] handler to call after this one succeeded.
///
/// # Returns
/// A [`Response`] given by the `next` handler, or a `401 NOT AUTHORIZED` if the user's login token did not check out.
pub async fn handle(
    State(state): State<ServerState>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    jar: PrivateCookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    info!("Middleware 'auth': inspecting client '{client}' login token");

    // Get the token first
    let token: Cookie = match jar.get(LOGIN_TOKEN_NAME) {
        Some(token) => token,
        None => {
            debug!("Client '{client}' did not provide any token; login failed");
            return Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::new(format!("No '{LOGIN_TOKEN_NAME}' cookie given"))).unwrap();
        },
    };
    debug!("Client '{}' provided token {:?}", client, token.value());

    // Run thru the checker
    let user: UserInfo = match check_token(&state.db, token.value()) {
        Ok(Ok(user)) => user,
        Ok(Err(err)) => {
            debug!("{}", trace!(("Client '{client}' provided an invalid token"), err));
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::new(format!("Invalid '{LOGIN_TOKEN_NAME}' cookie given")))
                .unwrap();
        },
        Err(err) => {
            error!("{}", trace!(("Failed to check login token {:?}", token.value()), err));
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::new(format!("Failed to check '{LOGIN_TOKEN_NAME}' cookie")))
                .unwrap();
        },
    };
    debug!("Client '{}' token {:?} OK", client, token.value());

    // Checks out, inject the result, then call the next middleware
    request.extensions_mut().insert(user);
    next.run(request).await
}
