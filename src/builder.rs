//! Incremental construction of an [`Ability`].

use crate::ability::Ability;
use crate::rule::{Effect, Rule};

/// Builds an [`Ability`] by accumulating unconditional rules.
///
/// Obtain one with [`Ability::builder`]. Rule methods take `&mut self` and
/// return `&mut Self`, so both chaining and conditional mutation read
/// naturally:
///
/// ```
/// # use praetor::Ability;
/// # #[derive(PartialEq, Eq)] enum Action { Read }
/// # #[derive(PartialEq, Eq)] enum Subject { Message }
/// let mut builder = Ability::builder(());
/// builder.can(Action::Read, Subject::Message);
/// let ability = builder.build();
/// ```
pub struct AbilityBuilder<A, S, C> {
    claims: Option<C>,
    rules: Vec<Rule<A, S>>,
}

impl<A, S, C> AbilityBuilder<A, S, C> {
    pub(crate) fn new(claims: C) -> Self {
        Self {
            claims: Some(claims),
            rules: Vec::new(),
        }
    }

    /// Returns the claims the ability is being built for.
    ///
    /// Applications use this to drive rule construction from authenticated
    /// state (roles, permissions, tenancy, ...).
    pub fn claims(&self) -> &C {
        self.claims.as_ref().expect("claims accessed after `build`")
    }

    /// Adds an unconditional allow rule for `action` on `subject`.
    pub fn can(&mut self, action: A, subject: S) -> &mut Self {
        self.rules.push(Rule {
            effect: Effect::Allow,
            action,
            subject,
        });
        self
    }

    /// Adds an unconditional deny rule for `action` on `subject`.
    pub fn cannot(&mut self, action: A, subject: S) -> &mut Self {
        self.rules.push(Rule {
            effect: Effect::Deny,
            action,
            subject,
        });
        self
    }

    /// Finalizes the accumulated rules into an [`Ability`].
    pub fn build(&mut self) -> Ability<A, S, C> {
        Ability::new(
            self.claims.take().expect("`build` called more than once"),
            std::mem::take(&mut self.rules),
        )
    }
}
