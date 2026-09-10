<p align="center">
  <img src="logo.png" alt="Praetor" width="160">
</p>

# Praetor

> Praetor is a small, strongly typed authorization library for Rust built around composable `can` and `cannot` rules.

[![crates.io](https://img.shields.io/crates/v/praetor.svg)](https://crates.io/crates/praetor)
[![docs.rs](https://img.shields.io/docsrs/praetor)](https://docs.rs/praetor)
[![license](https://img.shields.io/crates/l/praetor.svg)](LICENSE)

Praetor defines authorization *mechanics*, not your application's authorization
*model*. Actions, subjects, claims, roles, permissions, and JWT formats all stay
in your application. Praetor just composes rules, matches them, and returns a
decision.

```text
claims + rules + action + subject/resource  ->  allow / deny
```

## Install

```toml
[dependencies]
praetor = "0.1"
```

Praetor targets Rust **1.88+** and has no dependencies by default. The optional
Axum integration is behind the `axum` feature:

```toml
[dependencies]
praetor = { version = "0.1", features = ["axum"] }
```

## Quick start

```rust
use praetor::{Ability, AbilitySubject};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action { Read, Update }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Subject { Message }

struct Claims { user_id: u64 }

struct Message { author_id: u64, deleted: bool }

impl AbilitySubject<Subject> for Message {
    fn subject_type(&self) -> Subject { Subject::Message }
}

let ability = Ability::builder(Claims { user_id: 1 })
    .can(Action::Read, Subject::Message)
    .can_if(Action::Update, Subject::Message, |claims, m: &Message| {
        claims.user_id == m.author_id
    })
    .cannot_if(Action::Read, Subject::Message, |_, m: &Message| m.deleted)
    .build();

let mine = Message { author_id: 1, deleted: false };

// Subject-level check (no instance).
assert!(ability.can(Action::Read, Subject::Message));

// Instance-level check (predicate runs against the resource).
assert!(ability.can(Action::Update, &mine));

// `authorize` maps the same decision to a `Result`.
assert!(ability.authorize(Action::Update, &mine).is_ok());
```

## How it works

A single `can` / `authorize` interface accepts either a **logical subject**
(`Subject::Message`) or a **concrete resource** (`&message`). A resource type
opts in by implementing `AbilitySubject`, which maps it to its logical subject.

- **Unconditional rules** (`can` / `cannot`) match on action and subject.
- **Conditional rules** (`can_if` / `cannot_if`) additionally run a strongly
  typed closure `Fn(&C, &R) -> bool` against the resource instance. A
  conditional rule can only match when a concrete instance is supplied — a
  subject-only check never matches one.

Predicates are ordinary Rust closures. Heterogeneous resource types
(`Message`, `Room`, ...) are stored in one ability via internal type erasure
that is invisible to callers.

## Evaluation semantics

- **Default deny** — if no rule matches, access is denied.
- **Last matching rule wins** — later rules override earlier ones, so a
  `cannot_if` after a `can` can carve out an exception.

```rust
let ability = Ability::builder(())
    .can(Action::Read, Subject::Message)
    .cannot_if(Action::Read, Subject::Message, |_, m: &Message| m.deleted)
    .build();

let deleted = Message { deleted: true };
assert!(ability.cannot(Action::Read, &deleted)); // later deny wins
```

## What Praetor is not

Praetor deliberately does **not** provide:

- JWT validation, OAuth/OIDC, or a claims schema
- Built-in roles, permissions, or capability registries
- A policy DSL, predicate AST, or JSON condition format
- Serialized policies or remote/database-backed evaluation
- Built-in `Manage`/wildcard semantics

Authorization is a pure, in-memory operation. Load whatever state you need
first, then ask Praetor for a decision. Your application owns its model; Praetor
owns the mechanics.

## Links

- Website: <https://praetor.frazier.software>
- API docs: <https://docs.rs/praetor>
- Source: <https://github.com/chrisfrazier0/praetor>

## License

Licensed under the [MIT License](LICENSE).
