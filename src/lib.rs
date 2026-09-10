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
//! struct Message;
//! impl AbilitySubject<Subject> for Message {
//!     fn subject_type(&self) -> Subject { Subject::Message }
//! }
//!
//! let ability = Ability::builder(())
//!     .can(Action::Read, Subject::Message)
//!     .can(Action::Update, Subject::Message)
//!     .build();
//!
//! let message = Message;
//! assert!(ability.can(Action::Read, Subject::Message));
//! assert!(ability.can(Action::Update, &message));
//! assert!(ability.authorize(Action::Update, &message).is_ok());
//! ```
//!
//! Evaluation is **default deny**, and the **last matching rule wins**.

mod ability;
mod builder;
mod error;
mod rule;
mod subject;

pub use ability::Ability;
pub use builder::AbilityBuilder;
pub use error::Forbidden;
pub use rule::Effect;
pub use subject::{AbilitySubject, IntoTarget, ResourceTarget, SubjectTarget};

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
    }

    impl AbilitySubject<Subject> for Message {
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
        let message = Message { author_id: 1 };
        assert_eq!(message.author_id, claims().user_id);
        let ability = Ability::builder(claims())
            .can(Action::Update, Subject::Message)
            .build();
        assert!(ability.can(Action::Update, &message));
        assert!(ability.cannot(Action::Delete, &message));
    }

    #[test]
    fn authorize_maps_to_result() {
        let message = Message { author_id: 1 };
        let ability = Ability::builder(claims())
            .can(Action::Update, Subject::Message)
            .build();
        assert!(ability.authorize(Action::Update, &message).is_ok());
        assert_eq!(ability.authorize(Action::Delete, &message), Err(Forbidden));
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
