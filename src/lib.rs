//! TODO

#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![feature(arbitrary_self_types, const_fn_trait_bound)]
#![no_std]

use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

pub use regi_impl::*;

pub mod field;

pub mod mmio;

pub mod perms;

pub mod register;

/// Any integral type that can be used as for representation of underlying
/// storage of registers.
pub trait Int:
    Clone
    + Copy
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Not<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Default
    + PartialEq
    + sealed::Sealed
{
    /// The value of `0` for this type.
    const ZERO: Self;
}

macro_rules! impl_int {
    ($($ty:ty),*) => {
        $(
            impl Int for $ty {
                const ZERO: Self = 0;
            }
        )*
    };
}

impl_int!(u8, u16, u32, u64);

pub(crate) mod sealed {
    pub trait Sealed {}

    // Impls for supported register value primitives.
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}
