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

use core::{marker::PhantomData, ops};

use crate::{
    perms::{self, Permission},
    register::RegisterMarker,
    sealed::Sealed,
    Int,
};

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
pub struct Field<I, P, R> {
    pub(crate) mask: I,
    pub(crate) shift: usize,

    __perm: PhantomData<P>,
    __reg: PhantomData<R>,
}

/// A concrete value to be encoded into a register field.
///
/// Instances of this type should usually be obtained through
/// [`Field::make_value`].
#[derive(Clone, Copy, Debug)]
pub struct FieldValue<I, R> {
    mask: I,
    value: I,

    __reg: PhantomData<R>,
}

impl<I: Int, P: Permission, R: RegisterMarker> Field<I, P, R> {
    /// Constructs a new field given its encoding details.
    #[inline]
    pub const fn new(mask: I, shift: usize) -> Self {
        Self {
            mask,
            shift,

            __perm: PhantomData,
            __reg: PhantomData,
        }
    }

    /// Reads the specified bits of this field out of the given
    /// `value`.
    #[inline]
    pub fn read(self, value: I) -> I {
        self.select(value) >> self.shift
    }

    /// Selects the bits of this field in the given `value` and
    /// zeroes all other bits.
    ///
    /// If the desired behavior is to read out the bits of this
    /// field in a value, use [`Field::read`] instead.
    #[inline]
    pub fn select(self, value: I) -> I {
        value & (self.mask << self.shift)
    }

    /// Checks if the specified bits of this field are set in
    /// the given `value`.
    ///
    /// This returns `true` if *any* of the specified field bits
    /// are set, `false` otherwise.
    #[inline]
    pub fn is_set(self, value: I) -> bool {
        value & (self.mask << self.shift) != I::ZERO
    }
}

impl<I: Int, R: RegisterMarker> FieldValue<I, R> {
    /// Consumes the [`FieldValue`], returning the integer value it stores.
    #[inline]
    pub const fn into_inner(self) -> I {
        self.value
    }

    /// Encodes `new` into the wrapped value for the described field and
    /// returns the resulting updated value.
    #[inline]
    pub fn modify(self, new: I) -> I {
        (new & !self.mask) | self.mask
    }
}

macro_rules! impl_field_for {
    ($ty:ty) => {
        impl<P: Permission, R: RegisterMarker> Field<$ty, P, R> {
            /// Reads the specified bits of this field out of the given
            /// `value`.
            ///
            /// This does not rely on [`Int`] generics and can therefore
            /// be used in `const fn`s.
            #[inline]
            pub const fn const_read(self, value: $ty) -> $ty {
                (value & (self.mask << self.shift)) >> self.shift
            }

            /// Checks if the specified bits of this field are set in the
            /// given `value`.
            ///
            /// This returns `true` if *any* of the specified field bits are
            /// set, `false` otherwise.
            ///
            /// This does not rely on [`Int`] generics and can therefore
            /// be used in `const fn`s.
            #[inline]
            pub const fn const_is_set(self, value: $ty) -> bool {
                value & (self.mask << self.shift) != 0
            }

            /// Constructs a [`FieldValue`] from a concrete value, preserving
            /// the encoding information.
            ///
            /// This does not rely on [`Int`] generics and can therefore
            /// be used in `const fn`s.
            #[inline]
            pub const fn make_value(&self, value: $ty) -> FieldValue<$ty, R>
            where
                P: perms::Writable,
            {
                FieldValue::<$ty, R>::new(self.mask << self.shift, value)
            }
        }

        impl<R: RegisterMarker> FieldValue<$ty, R> {
            #[inline]
            pub(super) const fn new(mask: $ty, value: $ty) -> Self {
                Self {
                    mask,
                    value: value & mask,

                    __reg: PhantomData,
                }
            }

            /// Encodes `new` into the wrapped value for the described field
            /// and returns the resulting updated value.
            ///
            /// This does not rely on [`Int`] generics and can therefore
            /// be used in `const fn`s.
            #[inline]
            pub const fn const_modify(self, new: $ty) -> $ty {
                (new & !self.mask) | self.value
            }
        }

        /// Lowers a field value into the primitive it wraps.
        impl<R: RegisterMarker> From<FieldValue<$ty, R>> for $ty {
            #[inline]
            fn from(field: FieldValue<$ty, R>) -> Self {
                field.into_inner()
            }
        }

        /// Combine two field values using the `|` operator.
        impl<R: RegisterMarker> ops::BitOr<FieldValue<$ty, R>> for FieldValue<$ty, R> {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    mask: self.mask | rhs.mask,
                    value: self.value | rhs.value,

                    __reg: PhantomData,
                }
            }
        }

        /// Combine two field values using the `|=` operator.
        impl<R: RegisterMarker> ops::BitOrAssign<FieldValue<$ty, R>> for FieldValue<$ty, R> {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.mask |= rhs.mask;
                self.value |= rhs.value;
            }
        }

        /// Direct comparison with the integer value stored in a field.
        impl<R: RegisterMarker> PartialEq<$ty> for FieldValue<$ty, R> {
            fn eq(&self, rhs: &$ty) -> bool {
                self.value == *rhs
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

impl<I: Sealed + Copy, P, R> Clone for Field<I, P, R> {
    fn clone(&self) -> Self {
        Self {
            mask: self.mask,
            shift: self.shift,

            __perm: PhantomData,
            __reg: PhantomData,
        }
    }
}
impl<I: Sealed + Copy, P, R> Copy for Field<I, P, R> {}
