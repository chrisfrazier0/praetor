//! Subject resolution for the unified `can` / `authorize` target argument.

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

/// Resolves an authorization target into its subject `S`.
///
/// This is what lets a single `can` / `authorize` method accept either a
/// subject value (`Subject::Message`) or a resource reference (`&message`).
/// The `Marker` type parameter disambiguates the two blanket implementations
/// and is always inferred; consumers never name it.
pub trait IntoTarget<S, Marker> {
    /// Resolves this target into its subject.
    fn subject(self) -> S;
}

impl<S> IntoTarget<S, SubjectTarget> for S {
    fn subject(self) -> S {
        self
    }
}

impl<S, R> IntoTarget<S, ResourceTarget> for &R
where
    R: AbilitySubject<S>,
{
    fn subject(self) -> S {
        self.subject_type()
    }
}
