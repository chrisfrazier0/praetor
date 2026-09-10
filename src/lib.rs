//! Praetor is a small, strongly typed authorization library for Rust built
//! around composable `can` and `cannot` rules.
//!
//! Praetor defines authorization *mechanics*, not an application's
//! authorization *model*. Actions, subjects, claims, roles, permissions, and
//! JWT formats all belong to the consuming application.
//!
//! ```
//! use praetor::{Ability, AbilitySubject};
//!
//! #[derive(Clone, Copy, PartialEq, Eq)]
//! enum Action { Read, Update }
//!
//! #[derive(Clone, Copy, PartialEq, Eq)]
//! enum Subject { Message }
//!
//! struct Claims { user_id: u64 }
//!
//! struct Message { author_id: u64, deleted: bool }
//! impl AbilitySubject<Subject> for Message {
//!     fn subject_type(&self) -> Subject { Subject::Message }
//! }
//!
//! let ability = Ability::builder(Claims { user_id: 1 })
//!     .can(Action::Read, Subject::Message)
//!     .can_if(Action::Update, Subject::Message, |claims, m: &Message| {
//!         claims.user_id == m.author_id
//!     })
//!     .cannot_if(Action::Read, Subject::Message, |_, m: &Message| m.deleted)
//!     .build();
//!
//! let mine = Message { author_id: 1, deleted: false };
//! assert!(ability.can(Action::Read, Subject::Message)); // unconditional allow
//! assert!(ability.can(Action::Update, &mine));          // predicate: author matches
//! assert!(ability.authorize(Action::Update, &mine).is_ok());
//! ```
//!
//! Predicates are ordinary Rust closures. Evaluation is **default deny**, and
//! the **last matching rule wins**.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod ability;
mod builder;
mod error;
mod rule;
mod subject;

pub use ability::Ability;
pub use builder::AbilityBuilder;
pub use error::Forbidden;
pub use rule::Effect;
pub use subject::{AbilitySubject, IntoTarget, ResourceTarget, SubjectTarget, Target};

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Action {
        Read,
        Update,
        Delete,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Subject {
        Message,
        Room,
    }

    #[derive(Clone)]
    struct Claims {
        user_id: u64,
    }

    struct Message {
        author_id: u64,
        deleted: bool,
    }

    impl AbilitySubject<Subject> for Message {
        fn subject_type(&self) -> Subject {
            Subject::Message
        }
    }

    /// A distinct resource type that maps to the same subject as `Message`,
    /// used to exercise resource-type mismatch in conditional rules.
    struct Note;

    impl AbilitySubject<Subject> for Note {
        fn subject_type(&self) -> Subject {
            Subject::Message
        }
    }

    fn claims() -> Claims {
        Claims { user_id: 1 }
    }

    #[test]
    fn default_deny() {
        let ability = Ability::builder(claims()).build();
        assert!(ability.cannot(Action::Read, Subject::Message));
    }

    #[test]
    fn allow_then_deny_precedence() {
        let allow = Ability::builder(claims())
            .cannot(Action::Read, Subject::Message)
            .can(Action::Read, Subject::Message)
            .build();
        assert!(allow.can(Action::Read, Subject::Message));

        let deny = Ability::builder(claims())
            .can(Action::Read, Subject::Message)
            .cannot(Action::Read, Subject::Message)
            .build();
        assert!(deny.cannot(Action::Read, Subject::Message));
    }

    #[test]
    fn only_matching_action_and_subject() {
        let ability = Ability::builder(claims())
            .can(Action::Read, Subject::Message)
            .build();
        assert!(ability.cannot(Action::Update, Subject::Message));
        assert!(ability.cannot(Action::Read, Subject::Room));
    }

    #[test]
    fn instance_level_check() {
        let message = Message {
            author_id: 1,
            deleted: false,
        };
        assert_eq!(message.author_id, claims().user_id);
        let ability = Ability::builder(claims())
            .can(Action::Update, Subject::Message)
            .build();
        assert!(ability.can(Action::Update, &message));
        assert!(ability.cannot(Action::Delete, &message));
    }

    #[test]
    fn authorize_maps_to_result() {
        let message = Message {
            author_id: 1,
            deleted: false,
        };
        let ability = Ability::builder(claims())
            .can(Action::Update, Subject::Message)
            .build();
        assert!(ability.authorize(Action::Update, &message).is_ok());
        assert_eq!(ability.authorize(Action::Delete, &message), Err(Forbidden));
    }

    #[test]
    fn conditional_matches_only_with_instance_and_predicate() {
        let ability = Ability::builder(claims())
            .can_if(Action::Update, Subject::Message, |claims, m: &Message| {
                claims.user_id == m.author_id
            })
            .build();

        // No resource instance: a conditional rule cannot match.
        assert!(ability.cannot(Action::Update, Subject::Message));

        // Predicate satisfied.
        let mine = Message {
            author_id: 1,
            deleted: false,
        };
        assert!(ability.can(Action::Update, &mine));

        // Predicate not satisfied.
        let theirs = Message {
            author_id: 2,
            deleted: false,
        };
        assert!(ability.cannot(Action::Update, &theirs));
    }

    #[test]
    fn conditional_ignores_wrong_resource_type() {
        let ability = Ability::builder(claims())
            .can_if(Action::Update, Subject::Message, |_, _: &Message| true)
            .build();

        let message = Message {
            author_id: 1,
            deleted: false,
        };
        assert!(ability.can(Action::Update, &message));

        // `Note` maps to `Subject::Message` but is a different concrete type,
        // so the `Message` predicate must not match it.
        assert!(ability.cannot(Action::Update, &Note));
    }

    #[test]
    fn conditional_deny_overrides_earlier_allow() {
        let ability = Ability::builder(claims())
            .can(Action::Read, Subject::Message)
            .cannot_if(Action::Read, Subject::Message, |_, m: &Message| m.deleted)
            .build();

        let live = Message {
            author_id: 1,
            deleted: false,
        };
        let deleted = Message {
            author_id: 1,
            deleted: true,
        };

        assert!(ability.can(Action::Read, &live));
        assert!(ability.cannot(Action::Read, &deleted));
        // The conditional deny needs an instance, so the subject-only check
        // still falls through to the unconditional allow.
        assert!(ability.can(Action::Read, Subject::Message));
    }

    #[test]
    fn claims_accessible_on_builder_and_ability() {
        let mut builder = Ability::builder(claims());
        builder.can(Action::Read, Subject::Message);
        assert_eq!(builder.claims().user_id, 1);
        let ability = builder.build();
        assert_eq!(ability.claims().user_id, 1);
    }
}
