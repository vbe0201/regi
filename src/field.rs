//! Abstractions over bit fields inside registers.
//!
//! This module is centered around two important types:
//!
//! - [`Field`]s store *how* a field is encoded and statically provide only
//!   the APIs that are safe for access based on individual permissions.
//!
//! - [`FieldValue`] stores a concrete value along with the information on
//!   how to correctly encode said value into the register.
//! 
//! Both types can be used with supported unsigned primitive integer types
//! and permissions provided by [`crate::perms`].

use core::marker::PhantomData;

use crate::sealed::Sealed;

/// Pinpoints a specific bit field inside a register.
///
/// [`Field`]s store the position and width of a particular set of bits
/// to extract them from a register value.
///
/// Additionally, [`Field`]s are covariant over a type `P` which denotes
/// the access permissions the field is gated behind. Based on this detail
/// we statically only provide the APIs a register field is meant to be
/// interfaced with.
pub struct Field<I, P> {
    mask: I,
    shift: usize,

    __perm: PhantomData<P>,
}

pub struct FieldValue<I, P> {
    mask: I,
    value: I,

    __perm: PhantomData<P>,
}

impl<I: Sealed, P> Field<I, P> {
    pub const fn new(mask: I, shift: usize) -> Self {
        Self {
            mask,
            shift,

            __perm: PhantomData,
        }
    }
}

// `#[derive(Clone, Copy)]` does not produce the desired generic bounds.
// See: https://github.com/rust-lang/rust/issues/26925

impl<I: Sealed + Copy, P> Clone for Field<I, P> {
    fn clone(&self) -> Self {
        Self {
            mask: self.mask,
            shift: self.shift,

            __perm: PhantomData,
        }
    }
}
impl<I: Sealed + Copy, P> Copy for Field<I, P> {}

impl<I: Sealed + Copy, P> Clone for FieldValue<I, P> {
    fn clone(&self) -> Self {
        Self {
            mask: self.mask,
            value: self.value,

            __perm: PhantomData,
        }
    }
}
impl<I: Sealed + Copy, P> Copy for FieldValue<I, P> {}
