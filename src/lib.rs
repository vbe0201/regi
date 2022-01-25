//! TODO

#![deny(missing_docs, rustdoc::broken_intra_doc_links, unsafe_code)]
#![feature(const_fn_trait_bound)]
#![no_std]

pub mod field;

pub mod perms;

pub(crate) mod sealed {
    pub trait Sealed {}

    // Impls for supported register value primitives.
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
}
