//! Procedural macros for use with [`regi`].
//!
//! These macros are re-exported by the [`regi`] crate itself,
//! there is no need to explicitly add this crate to dependencies.
//!
//! [`regi`]: ../regi/

#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

mod ast;
mod expand;
