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

#[inline(always)]
const fn is_aligned(value: usize, align: usize) -> bool {
    debug_assert!(align.is_power_of_two());
    value & (align - 1) == 0
}

// Not part of the public API. Used by generated code.
// SAFETY: The pointer will be non-null and well-aligned.
#[doc(hidden)]
#[inline]
pub const fn register_block_ptr<T, I>(addr: usize) -> *mut T {
    assert!(addr != 0, "Address to register region must be non-zero!");
    assert!(
        is_aligned(addr, core::mem::size_of::<I>()),
        "Address must be aligned to the size of the first register!"
    );

    addr as *mut T
}

// Not part of the public API. Used by generated code.
#[doc(hidden)]
#[macro_export]
macro_rules! make_register_window {
    ($block:ident.$reg:ident) => {
        unsafe {
            let ptr = ::core::ptr::addr_of_mut!((*$block).$reg);
            $crate::mmio::RegisterWindow::new(ptr)
        }
    };
}

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
