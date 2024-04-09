//  AUTH.rs
//    by Lut99
//
//  Created:
//    08 Apr 2024, 11:36:08
//  Last edited:
//    09 Apr 2024, 13:02:31
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements tooling for doing user authentication, like password
//!   hashing.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString};
use argon2::Argon2;
use chrono::{DateTime, Utc};
use enum_debug::EnumDebug;
use error_trace::trace;
use log::debug;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::database::{Database, UserInfo};


/***** CONSTANTS *****/
/// The time that a token is valid.
pub const TOKEN_VALID_TIME_MIN: i64 = 360;

/// The name of the login token cookie.
pub const LOGIN_TOKEN_NAME: &'static str = "login-token";





/***** ERRORS *****/
/// Defines errors originating from parsing [`Role`]s from numbers.
#[derive(Debug)]
pub struct RoleFromU8Error(u8);
impl Display for RoleFromU8Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "Unknown role '{}'", self.0) }
}
impl Error for RoleFromU8Error {}



/// Defines errors originating from password hashing/checking.
#[derive(Debug)]
pub enum PasswordError {
    /// Failed to hash a given password.
    Hash { err: argon2::password_hash::Error },
}
impl Display for PasswordError {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use PasswordError::*;
        match self {
            Hash { .. } => write!(f, "Failed to hash password"),
        }
    }
}
impl Error for PasswordError {
    fn source(&self) -> Option<&(dyn 'static + Error)> {
        use PasswordError::*;
        match self {
            Hash { err } => Some(err),
        }
    }
}



/// Define errors originating from token managing/checking.
#[derive(Debug)]
pub enum TokenError {
    /// Failed to serialize the given login token.
    Serialize { err: serde_json::Error },
    /// Failed to get the info for a certain user.
    UserInfoRetrieve { id: u64, err: crate::database::Error },
}
impl Display for TokenError {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use TokenError::*;
        match self {
            Serialize { .. } => write!(f, "Failed to serialize login token"),
            UserInfoRetrieve { id, .. } => write!(f, "Failed to retrieve UserInfo for user {id} from database"),
        }
    }
}
impl Error for TokenError {
    #[inline]
    fn source(&self) -> Option<&(dyn 'static + Error)> {
        use TokenError::*;
        match self {
            Serialize { err } => Some(err),
            UserInfoRetrieve { err, .. } => Some(err),
        }
    }
}

/// Defines reasons why a given token is invalid.
#[derive(Debug)]
pub enum TokenInvalid {
    /// Failed to deserialize some string as a [`LoginToken`].
    Deserialize { raw: String, err: serde_json::Error },
    /// The given token has expired.
    Expired { id: u64, age: i64, valid_time: i64 },
    /// A token carried a role that didn't make sense.
    IncorrectRole { id: u64, got: Role, expected: Role },
    /// A user presented a token for a user that was deleted (or at least, not in the DB).
    UserNotFound { id: u64 },
}
impl Display for TokenInvalid {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use TokenInvalid::*;
        match self {
            Deserialize { raw, .. } => {
                write!(
                    f,
                    "Failed to deserialize raw string as login token\n\nRaw:\n{}\n{}\n{}\n",
                    (0..80).map(|_| '-').collect::<String>(),
                    raw,
                    (0..80).map(|_| '-').collect::<String>()
                )
            },
            Expired { id, age, valid_time } => {
                write!(f, "User {id} presented an expired token of {age} minutes old (limit is {valid_time} minutes)")
            },
            IncorrectRole { id, got, expected } => {
                write!(f, "User {id} role in token does not match role in database (got {}, expected {})", got.variant(), expected.variant())
            },
            UserNotFound { id } => write!(f, "User {id} in token not found"),
        }
    }
}
impl Error for TokenInvalid {
    #[inline]
    fn source(&self) -> Option<&(dyn 'static + Error)> {
        use TokenInvalid::*;
        match self {
            Deserialize { err, .. } => Some(err),
            Expired { .. } => None,
            IncorrectRole { .. } => None,
            UserNotFound { .. } => None,
        }
    }
}





/***** AUXILLARY *****/
/// Defines recognized user roles and ordering between them.
#[derive(Clone, Copy, Debug, Deserialize, EnumDebug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Role {
    /// It's the most powerful role.
    Root = 10,
}
impl From<Role> for u8 {
    #[inline]
    fn from(value: Role) -> Self {
        match value {
            Role::Root => 10,
        }
    }
}
impl TryFrom<u8> for Role {
    type Error = RoleFromU8Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(Self::Root),
            other => Err(RoleFromU8Error(other)),
        }
    }
}

/// The thing that we sent to users that acts as an auth token.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginToken {
    /// The ID of the logged-in user.
    pub id:     u64,
    /// The role of the logged-in user.
    pub role:   Role,
    /// The time this token was issued.
    pub issued: DateTime<Utc>,
}





/***** LIBRARY *****/
/// Computes the hash of a password.
///
/// # Arguments
/// - `password`: The password to hash.
///
/// # Returns
/// The hashed variant of the password, as a Base64-encoded string.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    // Generate a saltstring
    let salt: SaltString = SaltString::generate(&mut OsRng);

    // Hash
    match Argon2::default().hash_password(password.as_bytes(), &salt) {
        Ok(pass) => Ok(pass.to_string()),
        Err(err) => Err(PasswordError::Hash { err }),
    }
}

/// Compares the hash of a password with a plaintext suggestion.
///
/// # Arguments
/// - `password`: The given, plaintext password to compare.
/// - `hash`: The password in the DB to compare to.
///
/// # Returns
/// True if they are the same, or false if they aren't.
///
/// # Panics
/// This function panics if the given `hash` is not valid.
pub fn check_password(password: &str, hash: &str) -> bool {
    // Parse the hash, then compare
    let hash: PasswordHash = PasswordHash::new(hash).unwrap_or_else(|err| panic!("{}", trace!(("Illegal password hash '{hash}'"), err)));
    Argon2::default().verify_password(password.as_bytes(), &hash).is_ok()
}



/// Creates an opaque login string that can be sent to users to authorize them post-login.
///
/// # Arguments
/// - `id`: The identifier of the user for which the token is valid.
/// - `role`: The role of the user for which the token is valid.
///
/// # Returns
/// An already serialized string that embeds the token.
///
/// Note that this token is not signed. Instead, another method of encryption must be used (e.g., [`PrivateCookieJar`](axum_extra::extract::PrivateCookieJar)s).
///
/// # Errors
/// This function may error if we failed to serialize the token internally.
#[inline]
pub fn create_token(id: u64, role: Role) -> Result<String, TokenError> {
    match serde_json::to_string(&LoginToken { id, role, issued: Utc::now() }) {
        Ok(token) => Ok(token),
        Err(err) => Err(TokenError::Serialize { err }),
    }
}

/// Verifies if the given token is valid.
///
/// # Arguments
/// - `database`: A [`Database`] connection that we'll use to see if the user in the token exists.
/// - `token`: Some opaque string token that we will check.
///
/// # Returns
/// A [`UserInfo`] that describes the information of the logged-in user, or a [`TokenInvalid`] describing why the token was no longer valid.
///
/// # Errors
/// This function errors if we failed to use the given database.
#[inline]
pub fn check_token(database: &Database, token: &str) -> Result<Result<UserInfo, TokenInvalid>, TokenError> {
    match serde_json::from_str::<LoginToken>(token) {
        Ok(token) => {
            debug!("Got presented login token '{token:?}'");

            // First check if the token is still valid
            let age: i64 = (Utc::now() - token.issued).num_minutes();
            if age > TOKEN_VALID_TIME_MIN {
                // Assume not logged-in
                return Ok(Err(TokenInvalid::Expired { id: token.id, age, valid_time: TOKEN_VALID_TIME_MIN }));
            }

            // Then check if we can get the user from the database
            match database.get_user_by_id(token.id) {
                Ok(Some(user)) => {
                    // Finally, check if the role in the token is what we know of the user in the database
                    if user.role == token.role {
                        Ok(Ok(user))
                    } else {
                        Ok(Err(TokenInvalid::IncorrectRole { id: user.id, got: token.role, expected: user.role }))
                    }
                },
                Ok(None) => Ok(Err(TokenInvalid::UserNotFound { id: token.id })),
                Err(err) => Err(TokenError::UserInfoRetrieve { id: token.id, err }),
            }
        },
        Err(err) => Ok(Err(TokenInvalid::Deserialize { raw: token.into(), err })),
    }
}
