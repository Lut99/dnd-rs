//  SPEC.rs
//    by Lut99
//
//  Created:
//    09 Apr 2024, 12:15:18
//  Last edited:
//    09 Apr 2024, 12:16:22
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some definitions for the spec of the server.
//

use hyper::Method;


/***** LIBRARY *****/
/// Defines how a path definition looks like.
pub struct Path {
    /// The HTTP method used to access the path.
    pub method: Method,
    /// The path on which the method can be found.
    pub path:   &'static str,
}
