//! Authorization error type.

use std::error::Error;
use std::fmt;

/// Returned by [`Ability::authorize`](crate::Ability::authorize) when a request
/// is denied.
///
/// The error is intentionally opaque and framework-independent: it carries no
/// rule or claims details. Applications map it to whatever they need, such as
/// HTTP 403.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Forbidden;

impl fmt::Display for Forbidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("authorization denied")
    }
}

impl Error for Forbidden {}
