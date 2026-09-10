//! Axum integration (enabled by the `axum` feature).
//!
//! Praetor never validates JWTs or authenticates requests. The application's
//! own middleware is responsible for validating the request, building an
//! [`Ability`](crate::Ability), and inserting it into the request extensions as
//! an `Arc`:
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use axum::{extract::Request, middleware::Next, response::Response};
//! # type AppAbility = praetor::Ability<(), (), ()>;
//! # fn build_ability() -> AppAbility { praetor::Ability::builder(()).build() }
//! async fn auth(mut request: Request, next: Next) -> Response {
//!     let ability: AppAbility = build_ability(); // after validating the JWT
//!     request.extensions_mut().insert(Arc::new(ability));
//!     next.run(request).await
//! }
//! ```
//!
//! Handlers then retrieve it with the [`Praetor`] extractor:
//!
//! ```no_run
//! # use praetor::axum::Praetor;
//! # type AppAbility = praetor::Ability<(), (), ()>;
//! async fn handler(Praetor(ability): Praetor<AppAbility>) {
//!     // `ability` is an `Arc<AppAbility>`; it derefs to the ability, so
//!     // `ability.authorize(..)` works directly.
//! }
//! ```

use std::ops::Deref;
use std::sync::Arc;

use ::axum::extract::FromRequestParts;
use ::axum::http::StatusCode;
use ::axum::http::request::Parts;
use ::axum::response::{IntoResponse, Response};

/// Extractor that retrieves a shared `T` (typically an
/// [`Ability`](crate::Ability)) previously inserted into the request extensions
/// as `Arc<T>`.
///
/// The value is shared, not cloned: extraction clones the `Arc` only. Destructure
/// it to obtain the inner `Arc<T>`, which derefs to `T`:
///
/// ```no_run
/// # use praetor::axum::Praetor;
/// # type AppAbility = praetor::Ability<(), (), ()>;
/// async fn handler(Praetor(ability): Praetor<AppAbility>) {
///     // `ability.authorize(..)` and other `Ability` methods work via deref.
/// }
/// ```
pub struct Praetor<T>(pub Arc<T>);

impl<T> Deref for Praetor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Rejection returned by [`Praetor`] when no `Arc<T>` is present in the request
/// extensions.
///
/// This indicates the ability was never inserted — usually a misconfiguration
/// where the middleware that builds the ability did not run. It converts into a
/// `500 Internal Server Error`. Applications that prefer a different status can
/// insert the ability unconditionally, or wrap the extractor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingAbility;

impl std::fmt::Display for MissingAbility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ability not found in request extensions")
    }
}

impl std::error::Error for MissingAbility {}

impl IntoResponse for MissingAbility {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl<S, T> FromRequestParts<S> for Praetor<T>
where
    S: Send + Sync,
    T: Send + Sync + 'static,
{
    type Rejection = MissingAbility;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Arc<T>>()
            .cloned()
            .map(Praetor)
            .ok_or(MissingAbility)
    }
}
