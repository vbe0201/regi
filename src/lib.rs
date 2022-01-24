//! TODO

#![deny(rustdoc::broken_intra_doc_links, unsafe_code)]
#![feature(const_fn_trait_bound)]
#![no_std]

pub mod field;

pub mod perms;

pub(crate) mod sealed {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}
