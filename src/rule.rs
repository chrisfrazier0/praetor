//! Internal rule representation.

use std::any::Any;

/// Whether a matching rule grants or denies access.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    /// The rule grants access.
    Allow,
    /// The rule denies access.
    Deny,
}

/// A type-erased conditional predicate.
///
/// It receives the ability's claims and the concrete resource (as `&dyn Any`)
/// and reports whether the rule applies. The erasure lets a single ability
/// hold predicates against many different resource types while the
/// construction API stays strongly typed.
pub(crate) type Predicate<C> = dyn Fn(&C, &dyn Any) -> bool + Send + Sync;

/// A single authorization rule.
///
/// Unconditional rules (`condition == None`) match on `action`/`subject`
/// equality alone. Conditional rules additionally require a concrete resource
/// instance whose type matches the predicate's expected resource and for which
/// the predicate returns `true`.
pub(crate) struct Rule<A, S, C> {
    pub(crate) effect: Effect,
    pub(crate) action: A,
    pub(crate) subject: S,
    pub(crate) condition: Option<Box<Predicate<C>>>,
}

impl<A, S, C> Rule<A, S, C> {
    pub(crate) fn unconditional(effect: Effect, action: A, subject: S) -> Self {
        Self {
            effect,
            action,
            subject,
            condition: None,
        }
    }

    pub(crate) fn conditional<R, F>(effect: Effect, action: A, subject: S, predicate: F) -> Self
    where
        R: 'static,
        F: Fn(&C, &R) -> bool + Send + Sync + 'static,
    {
        let predicate: Box<Predicate<C>> =
            Box::new(move |claims, resource| match resource.downcast_ref::<R>() {
                Some(resource) => predicate(claims, resource),
                None => false,
            });
        Self {
            effect,
            action,
            subject,
            condition: Some(predicate),
        }
    }
}
