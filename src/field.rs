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

use crate::{perms::*, sealed::Sealed};

/// Pinpoints a specific bit field inside a register.
///
/// [`Field`]s store the position and width of a particular set of bits
/// to extract them from a register value.
///
/// Additionally, [`Field`]s are covariant over a type `P` which denotes
/// the access permissions the field is gated behind. Based on this detail
/// we statically only provide the APIs a register field is meant to be
/// interfaced with.
#[derive(Debug)]
pub struct Field<I, P> {
    mask: I,
    shift: usize,

    __perm: PhantomData<P>,
}

/// A concrete value to be encoded into a register field.
///
/// Instances of this type should usually be obtained through
/// [`Field::make_value`].
#[derive(Clone, Copy, Debug)]
pub struct FieldValue<I> {
    mask: I,
    value: I,
}

impl<I: Sealed, P> Field<I, P> {
    /// Constructs a new field given its encoding details.
    #[inline]
    pub const fn new(mask: I, shift: usize) -> Self {
        Self {
            mask,
            shift,

            __perm: PhantomData,
        }
    }
}

macro_rules! impl_field_for {
    ($ty:ty) => {
        impl<P: Permission> Field<$ty, P> {
            /// Reads the specified bits of this field out of the given
            /// `value`.
            #[inline]
            pub const fn read(self, value: $ty) -> $ty {
                (value & (self.mask << self.shift)) >> self.shift
            }

            /// Checks if the specified bits of this field are set in the
            /// given `value`.
            ///
            /// This returns `true` if *any* of the specified field bits are
            /// set, `false` otherwise.
            #[inline]
            pub const fn is_set(self, value: $ty) -> bool {
                value & (self.mask << self.shift) != 0
            }

            /// Constructs a [`FieldValue`] from a concrete value, preserving
            /// the encoding information.
            #[inline]
            pub const fn make_value(&self, value: $ty) -> FieldValue<$ty> {
                FieldValue::<$ty>::new(self.mask << self.shift, value)
            }
        }

        impl FieldValue<$ty> {
            #[inline]
            pub(super) const fn new(mask: $ty, value: $ty) -> Self {
                Self {
                    mask,
                    value: value & mask,
                }
            }

            /// Encodes `new` into the wrapped value for the described field
            /// and returns the resulting updated value.
            #[inline]
            pub const fn modify(self, new: $ty) -> $ty {
                (new & !self.mask) | self.value
            }
        }

        /// Lowers a field value into the primitive it wraps.
        impl From<FieldValue<$ty>> for $ty {
            fn from(field: FieldValue<$ty>) -> Self {
                field.value
            }
        }
    };
}

impl_field_for!(u8);
impl_field_for!(u16);
impl_field_for!(u32);
impl_field_for!(u64);

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
