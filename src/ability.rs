//! The authorization ability and its evaluation.

use crate::builder::AbilityBuilder;
use crate::error::Forbidden;
use crate::rule::{Effect, Rule};
use crate::subject::{IntoTarget, Target};

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
    rules: Vec<Rule<A, S, C>>,
}

impl<A, S, C> Ability<A, S, C> {
    pub(crate) fn new(claims: C, rules: Vec<Rule<A, S, C>>) -> Self {
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
    /// [`AbilitySubject`](crate::AbilitySubject). Conditional rules only match
    /// when a resource instance is supplied.
    pub fn can<'t, T, M>(&self, action: A, target: T) -> bool
    where
        A: PartialEq,
        S: PartialEq,
        T: IntoTarget<'t, S, M>,
    {
        self.evaluate(&action, target.into_target())
    }

    /// Returns whether `action` is *not* permitted on `target`.
    ///
    /// Convenience negation of [`can`](Self::can).
    pub fn cannot<'t, T, M>(&self, action: A, target: T) -> bool
    where
        A: PartialEq,
        S: PartialEq,
        T: IntoTarget<'t, S, M>,
    {
        !self.can(action, target)
    }

    /// Like [`can`](Self::can) but returns [`Forbidden`] when denied.
    pub fn authorize<'t, T, M>(&self, action: A, target: T) -> Result<(), Forbidden>
    where
        A: PartialEq,
        S: PartialEq,
        T: IntoTarget<'t, S, M>,
    {
        if self.can(action, target) {
            Ok(())
        } else {
            Err(Forbidden)
        }
    }

    fn evaluate(&self, action: &A, target: Target<'_, S>) -> bool
    where
        A: PartialEq,
        S: PartialEq,
    {
        for rule in self.rules.iter().rev() {
            if &rule.action != action || rule.subject != target.subject {
                continue;
            }
            match &rule.condition {
                // Unconditional rules match subject-only and instance checks.
                None => return rule.effect == Effect::Allow,
                // Conditional rules require a concrete resource instance whose
                // type and predicate satisfy the stored closure.
                Some(predicate) => {
                    if let Some(resource) = target.resource
                        && predicate(&self.claims, resource)
                    {
                        return rule.effect == Effect::Allow;
                    }
                }
            }
        }
        false
    }
}
