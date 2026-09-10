//! End-to-end Axum integration (requires the `axum` feature).
//!
//! Flow: JWT -> Claims -> Ability -> request extension -> handler -> authorize.
//! Praetor never validates the JWT; the middleware does that and inserts the
//! ability. This example drives the router in-process with `tower`'s `oneshot`
//! so it runs to completion without binding a port.
//!
//! Run with: `cargo run --example axum --features axum`

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use praetor::axum::Praetor;
use praetor::{Ability, AbilitySubject};
use tower::ServiceExt;

// --- Application-defined authorization model ---------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Read,
    Update,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Subject {
    Message,
}

#[derive(Clone)]
struct Claims {
    user_id: u64,
}

#[derive(Clone)]
struct Message {
    id: u64,
    author_id: u64,
    body: String,
}

impl AbilitySubject<Subject> for Message {
    fn subject_type(&self) -> Subject {
        Subject::Message
    }
}

type AppAbility = Ability<Action, Subject, Claims>;

#[derive(Clone)]
struct AppState {
    messages: Arc<HashMap<u64, Message>>,
}

/// Built from validated claims. Anyone may read messages; only the author may
/// update their own.
fn build_ability(claims: Claims) -> AppAbility {
    Ability::builder(claims)
        .can(Action::Read, Subject::Message)
        .can_if(Action::Update, Subject::Message, |claims, m: &Message| {
            claims.user_id == m.author_id
        })
        .build()
}

/// Stand-in for real auth middleware: validate the JWT, decode claims, build the
/// ability, and insert it as an `Arc` into the request extensions.
async fn auth(mut request: Request, next: Next) -> Response {
    let claims = Claims { user_id: 1 }; // pretend this came from a validated JWT
    request
        .extensions_mut()
        .insert(Arc::new(build_ability(claims)));
    next.run(request).await
}

/// Application error mapped to HTTP responses. Praetor's `Forbidden` maps to 403.
enum ApiError {
    NotFound,
    Forbidden,
}

impl From<praetor::Forbidden> for ApiError {
    fn from(_: praetor::Forbidden) -> Self {
        ApiError::Forbidden
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found").into_response(),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "forbidden").into_response(),
        }
    }
}

async fn update_message(
    Praetor(ability): Praetor<AppAbility>,
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<String, ApiError> {
    let message = state.messages.get(&id).cloned().ok_or(ApiError::NotFound)?;
    ability.authorize(Action::Update, &message)?;
    Ok(format!("updated message {}: {}", message.id, message.body))
}

fn app() -> Router {
    let messages = HashMap::from([
        (
            1,
            Message {
                id: 1,
                author_id: 1,
                body: "mine".into(),
            },
        ),
        (
            2,
            Message {
                id: 2,
                author_id: 2,
                body: "theirs".into(),
            },
        ),
    ]);
    let state = AppState {
        messages: Arc::new(messages),
    };

    Router::new()
        .route("/messages/{id}", get(update_message))
        .layer(middleware::from_fn(auth))
        .with_state(state)
}

async fn call(app: Router, id: u64) -> (StatusCode, String) {
    let request = Request::builder()
        .uri(format!("/messages/{id}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::main]
async fn main() {
    // The authenticated user (id 1) owns message 1 but not message 2.
    let (status, body) = call(app(), 1).await;
    println!("update own message:    {status} {body}");

    let (status, body) = call(app(), 2).await;
    println!("update others message: {status} {body}");
}
