//! The authorization ability and its evaluation.

use crate::builder::AbilityBuilder;
use crate::error::Forbidden;
use crate::rule::{Effect, Rule};
use crate::subject::IntoTarget;

/// An immutable set of authorization rules evaluated against a fixed set of
/// claims.
///
/// Evaluation semantics:
///
/// - The default decision is **deny** when no rule matches.
/// - The **last matching rule wins**, so later rules override earlier ones.
///
/// `Ability` owns its claims `C` for simple movement between layers (for
/// example, into an Axum request extension).
pub struct Ability<A, S, C> {
    claims: C,
    rules: Vec<Rule<A, S>>,
}

impl<A, S, C> Ability<A, S, C> {
    pub(crate) fn new(claims: C, rules: Vec<Rule<A, S>>) -> Self {
        Self { claims, rules }
    }

    /// Starts building an ability for the given claims.
    pub fn builder(claims: C) -> AbilityBuilder<A, S, C> {
        AbilityBuilder::new(claims)
    }

    /// Returns the claims this ability was built for.
    pub fn claims(&self) -> &C {
        &self.claims
    }

    /// Returns whether `action` is permitted on `target`.
    ///
    /// `target` may be a subject value (`Subject::Message`) or a resource
    /// reference (`&message`) whose type implements
    /// [`AbilitySubject`](crate::AbilitySubject).
    pub fn can<T, M>(&self, action: A, target: T) -> bool
    where
        A: PartialEq,
        S: PartialEq,
        T: IntoTarget<S, M>,
    {
        let subject = target.subject();
        self.evaluate(&action, &subject)
    }

    /// Returns whether `action` is *not* permitted on `target`.
    ///
    /// Convenience negation of [`can`](Self::can).
    pub fn cannot<T, M>(&self, action: A, target: T) -> bool
    where
        A: PartialEq,
        S: PartialEq,
        T: IntoTarget<S, M>,
    {
        !self.can(action, target)
    }

    /// Like [`can`](Self::can) but returns [`Forbidden`] when denied.
    pub fn authorize<T, M>(&self, action: A, target: T) -> Result<(), Forbidden>
    where
        A: PartialEq,
        S: PartialEq,
        T: IntoTarget<S, M>,
    {
        if self.can(action, target) {
            Ok(())
        } else {
            Err(Forbidden)
        }
    }

    fn evaluate(&self, action: &A, subject: &S) -> bool
    where
        A: PartialEq,
        S: PartialEq,
    {
        for rule in self.rules.iter().rev() {
            if &rule.action == action && &rule.subject == subject {
                return rule.effect == Effect::Allow;
            }
        }
        false
    }
}
