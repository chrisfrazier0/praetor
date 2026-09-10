# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

Initial release.

### Added

- `Ability<A, S, C>`: an immutable, owned set of authorization rules evaluated
  against a fixed set of claims.
- `AbilityBuilder<A, S, C>` via `Ability::builder(claims)`, with chainable and
  `&mut self` mutation-friendly rule construction.
- Unconditional rules: `can(action, subject)` and `cannot(action, subject)`.
- Conditional rules: `can_if(action, subject, predicate)` and
  `cannot_if(action, subject, predicate)` with strongly typed
  `Fn(&C, &R) -> bool` predicates. Heterogeneous resource types are stored in a
  single ability via internal type erasure that never appears in the public API.
- A single unified `can` / `authorize` interface accepting either a logical
  subject value or a concrete resource reference.
- `AbilitySubject<S>` trait for mapping a concrete resource type to its logical
  subject, enabling `ability.can(action, &resource)`.
- `authorize` returning `Result<(), Forbidden>`; `Forbidden` is an opaque,
  framework-independent error.
- `claims()` accessor on both the builder and the built ability.
- Evaluation semantics: **default deny** and **last matching rule wins**.
  Conditional rules match only when a concrete resource instance is supplied.

[Unreleased]: https://github.com/chrisfrazier0/praetor/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/chrisfrazier0/praetor/releases/tag/v0.1.0
