//  LIB.rs
//    by Lut99
//
//  Created:
//    06 Apr 2024, 15:25:37
//  Last edited:
//    09 Apr 2024, 12:15:53
//  Auto updated?
//    Yes
//
//  Description:
//!   A server that hosts a website to play DnD with your friends!
//!   
//!   This part is the library of the server, which re-exports its feature for
//!   use in other Rust projects.
//

// Declare modules
pub mod auth;
pub mod database;
pub mod middleware;
pub mod paths;
pub mod spec;
pub mod state;
