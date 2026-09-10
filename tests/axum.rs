//! Integration tests for the optional Axum extractor.
//!
//! Run with: `cargo test --features axum`

#![cfg(feature = "axum")]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{FromRequestParts, Request};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use praetor::axum::{MissingAbility, Praetor};
use praetor::{Ability, AbilitySubject};
use tower::ServiceExt;

#[tokio::test]
async fn extracts_present_value() {
    let request = Request::builder()
        .extension(Arc::new(42u32))
        .body(())
        .unwrap();
    let (mut parts, ()) = request.into_parts();

    let Praetor(value) = Praetor::<u32>::from_request_parts(&mut parts, &())
        .await
        .expect("value should be present");
    assert_eq!(*value, 42);
}

#[tokio::test]
async fn missing_value_is_rejected() {
    let request = Request::builder().body(()).unwrap();
    let (mut parts, ()) = request.into_parts();

    let result = Praetor::<u32>::from_request_parts(&mut parts, &()).await;
    assert_eq!(result.err(), Some(MissingAbility));
}

// --- End-to-end router behavior ---------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Read,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Subject {
    Message,
}

struct Message {
    author_id: u64,
}

impl AbilitySubject<Subject> for Message {
    fn subject_type(&self) -> Subject {
        Subject::Message
    }
}

type AppAbility = Ability<Action, Subject, u64>;

enum ApiError {
    Forbidden,
}

impl From<praetor::Forbidden> for ApiError {
    fn from(_: praetor::Forbidden) -> Self {
        ApiError::Forbidden
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        StatusCode::FORBIDDEN.into_response()
    }
}

async fn auth(mut request: Request, next: Next) -> Response {
    let ability: AppAbility = Ability::builder(1u64)
        .can_if(Action::Read, Subject::Message, |uid, m: &Message| {
            *uid == m.author_id
        })
        .build();
    request.extensions_mut().insert(Arc::new(ability));
    next.run(request).await
}

async fn read(Praetor(ability): Praetor<AppAbility>) -> Result<String, ApiError> {
    // The authenticated user is id 1; this resource is owned by id 2.
    ability.authorize(Action::Read, &Message { author_id: 2 })?;
    Ok("ok".into())
}

async fn status_of(app: Router) -> StatusCode {
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    app.oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn forbidden_maps_to_403() {
    let app = Router::new()
        .route("/", get(read))
        .layer(middleware::from_fn(auth));
    assert_eq!(status_of(app).await, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn missing_ability_maps_to_500() {
    // No auth middleware, so nothing inserts the ability.
    let app = Router::new().route("/", get(read));
    let (status, body) = {
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    };
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body.contains("ability not found"));
}
