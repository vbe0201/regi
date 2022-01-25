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

/// Specialized trait to mark permissions which grant read access to a
/// [`Field`][crate::field::Field].
pub trait Readable: Permission {}

/// Specialized trait to mark permissions which grant write access to a
/// [`Field`][crate::field::Field].
pub trait Writable: Permission {}

/// A union of two permissions `P1` and `P2` to determine the new minimum
/// permission type that is reconcilable with both.
pub trait PermissionUnion<P1: Permission, P2: Permission> {
    /// The resulting permission type.
    type Union: Permission;
}

/// Permission marker to tag read-only register fields.
pub struct ReadOnly;
impl Sealed for ReadOnly {}
impl Permission for ReadOnly {}
impl Readable for ReadOnly {}

/// Permission marker to tag write-only register fields.
pub struct WriteOnly;
impl Sealed for WriteOnly {}
impl Permission for WriteOnly {}
impl Writable for WriteOnly {}

/// Permission marker to tag both readable and writable register fields.
pub struct ReadWrite;
impl Sealed for ReadWrite {}
impl Permission for ReadWrite {}
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

impl<R: Readable> PermissionUnion<ReadOnly, R> for ReadOnly {
    type Union = ReadOnly;
}

impl<W: Writable> PermissionUnion<WriteOnly, W> for WriteOnly {
    type Union = WriteOnly;
}

impl<P: Permission> PermissionUnion<ReadWrite, P> for ReadWrite {
    type Union = P;
}
