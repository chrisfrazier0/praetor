//! Subject resolution for the unified `can` / `authorize` target argument.

use std::any::Any;

/// Associates a concrete resource type with its logical subject `S`.
///
/// Implementing this lets a resource instance be passed directly to
/// [`Ability::can`](crate::Ability::can) without separately naming its subject:
///
/// ```
/// # use praetor::AbilitySubject;
/// #[derive(PartialEq, Eq)]
/// enum Subject { Message }
///
/// struct Message;
///
/// impl AbilitySubject<Subject> for Message {
///     fn subject_type(&self) -> Subject {
///         Subject::Message
///     }
/// }
/// ```
pub trait AbilitySubject<S> {
    /// Returns the logical subject this resource belongs to.
    fn subject_type(&self) -> S;
}

/// Marker selecting the subject-value branch of [`IntoTarget`].
pub struct SubjectTarget;

/// Marker selecting the resource-reference branch of [`IntoTarget`].
pub struct ResourceTarget;

/// An authorization target resolved to its subject, optionally carrying the
/// concrete resource instance for conditional-rule evaluation.
///
/// This type is opaque: any resource instance is stored type-erased internally
/// and is never exposed to consumers.
pub struct Target<'a, S> {
    pub(crate) subject: S,
    pub(crate) resource: Option<&'a dyn Any>,
}

/// Resolves an authorization target into a [`Target`].
///
/// This is what lets a single `can` / `authorize` method accept either a
/// subject value (`Subject::Message`) or a resource reference (`&message`). A
/// subject value resolves to a [`Target`] with no resource, so conditional
/// rules cannot match it; a resource reference additionally carries the
/// instance so conditional predicates can be evaluated against it.
///
/// The `Marker` type parameter disambiguates the two blanket implementations
/// and is always inferred; consumers never name it.
pub trait IntoTarget<'a, S, Marker> {
    /// Resolves this value into a [`Target`].
    fn into_target(self) -> Target<'a, S>;
}

impl<'a, S> IntoTarget<'a, S, SubjectTarget> for S {
    fn into_target(self) -> Target<'a, S> {
        Target {
            subject: self,
            resource: None,
        }
    }
}

impl<'a, S, R> IntoTarget<'a, S, ResourceTarget> for &'a R
where
    R: AbilitySubject<S> + 'static,
{
    fn into_target(self) -> Target<'a, S> {
        Target {
            subject: self.subject_type(),
            resource: Some(self),
        }
    }
}
