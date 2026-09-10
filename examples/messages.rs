//! A full authorization flow, framework-free.
//!
//! This mirrors how an application would use Praetor after it has already
//! validated a JWT and decoded its claims: build an `Ability` from those
//! claims, then authorize both subject-level and instance-level requests.
//!
//! Run with: `cargo run --example messages`

use std::collections::HashSet;

use praetor::{Ability, AbilitySubject};

// --- Application-defined authorization model ---------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Read,
    Update,
    Delete,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Subject {
    Message,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Permission {
    ReadMessages,
    EditOwnMessages,
}

/// The claims an application would decode from a validated JWT.
struct Claims {
    user_id: u64,
    permissions: HashSet<Permission>,
}

/// A concrete resource loaded from storage before authorization.
struct Message {
    author_id: u64,
    deleted: bool,
    system: bool,
}

impl AbilitySubject<Subject> for Message {
    fn subject_type(&self) -> Subject {
        Subject::Message
    }
}

// --- Ability construction ----------------------------------------------------

/// Builds an ability from authenticated claims. This logic belongs to the
/// application: Praetor only evaluates the rules it is given.
fn build_ability(claims: Claims) -> Ability<Action, Subject, Claims> {
    let mut ability = Ability::builder(claims);

    if ability
        .claims()
        .permissions
        .contains(&Permission::ReadMessages)
    {
        ability.can(Action::Read, Subject::Message);
    }

    if ability
        .claims()
        .permissions
        .contains(&Permission::EditOwnMessages)
    {
        ability.can_if(Action::Update, Subject::Message, |claims, m: &Message| {
            claims.user_id == m.author_id
        });
    }

    // A later rule overrides earlier ones: deleted messages are never readable,
    // and system messages are never deletable, regardless of other grants.
    ability.cannot_if(Action::Read, Subject::Message, |_, m: &Message| m.deleted);
    ability.cannot_if(Action::Delete, Subject::Message, |_, m: &Message| m.system);

    ability.build()
}

fn main() {
    let claims = Claims {
        user_id: 1,
        permissions: HashSet::from([Permission::ReadMessages, Permission::EditOwnMessages]),
    };

    let ability = build_ability(claims);

    // Subject-level check: an unconditional grant, no instance required.
    println!(
        "read messages:         {}",
        ability.can(Action::Read, Subject::Message)
    );

    let mine = Message {
        author_id: 1,
        deleted: false,
        system: false,
    };
    let theirs = Message {
        author_id: 2,
        deleted: false,
        system: false,
    };
    let deleted = Message {
        author_id: 1,
        deleted: true,
        system: false,
    };

    // Instance-level checks: the conditional predicate runs against the message.
    println!(
        "update own message:    {}",
        ability.can(Action::Update, &mine)
    );
    println!(
        "update others message: {}",
        ability.can(Action::Update, &theirs)
    );

    // `last matching rule wins`: the `cannot_if` deny overrides the read grant.
    println!(
        "read deleted message:  {}",
        ability.can(Action::Read, &deleted)
    );

    // `authorize` maps the decision to a `Result` for `?`-style flows.
    match ability.authorize(Action::Update, &mine) {
        Ok(()) => println!("authorize update own:  allowed"),
        Err(err) => println!("authorize update own:  {err}"),
    }
    match ability.authorize(Action::Delete, &mine) {
        Ok(()) => println!("authorize delete own:  allowed"),
        Err(err) => println!("authorize delete own:  {err}"),
    }
}
