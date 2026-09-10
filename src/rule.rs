//! Internal rule representation.

/// Whether a matching rule grants or denies access.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    /// The rule grants access.
    Allow,
    /// The rule denies access.
    Deny,
}

/// A single authorization rule.
///
/// For Milestone 1 rules are unconditional: they match purely on `action` and
/// `subject` equality. Conditional predicates are introduced in a later
/// milestone.
pub(crate) struct Rule<A, S> {
    pub(crate) effect: Effect,
    pub(crate) action: A,
    pub(crate) subject: S,
}
