//! Permission states for gating access to [`Field`][crate::field::Field]s
//! inside registers.
//!
//! Permissions for fields should be chosen based on individual requirements
//! stated in the Technical Reference Manual for the implemented registers.

use crate::sealed::Sealed;

/// Supertrait for all permissions provided by the [`crate::perms`] module.
///
/// [`Field`][crate::field::Field] and [`FieldValue`][crate::field::FieldValue]
/// are made covariant over particular [`Permission`]s to prevent misuse of
/// APIs when applied incorrectly.
pub trait Permission: Sealed {}
