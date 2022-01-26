//! Unified APIs for driving access to MMIO and CPU registers.
//!
//! The traits from this module outline how registers can be
//! interacted with, while paying respect to their access
//! permissions.

use crate::sealed::Sealed;

/// A marker type that is used to associate bit fields with registers.
///
/// This prevents runtime register access with unwanted bit fields that
/// were not declared along with the register at compile-time.
pub trait RegisterMarker {}

/// The unit type works for registers which don't actually have bit
/// fields defined for them.
impl RegisterMarker for () {}

/// Defines read access to MMIO and CPU registers.
///
/// Users may implement this trait for their own eligible types.
///
/// # Safety
///
/// This trait must only be implemented for register types that are
/// tagged [`Readable`][crate::perms::Readable] in accordance with the
/// Technical Reference Manual for the respective device.
pub unsafe trait RegisterRead {
    /// The primitive integer type that represents the storage unit of
    /// the underlying register.
    type Register: Sealed + Copy;

    /// Reads the raw value from the register.
    ///
    /// # A note on `&mut self`
    ///
    /// Many crates take `&self` when accessing registers. We would
    /// like to object with this position.
    ///
    /// In reality, there are many cases where register reads are not
    /// only done to obtain values, but also to commit a previous
    /// write or making sure hardware state changes are applied in
    /// reasonable time.
    ///
    /// From the perspective of a Rust API, we want to be very explicit
    /// about such aspects and don't want to hide the invariant of read
    /// operations mutating hardware state behind what is portrayed
    /// immutable.
    ///
    /// # Safety
    ///
    /// Access permissions to the individual bit fields of the register
    /// are not enforced. The user needs to make sure to operate on the
    /// received value in accordance with the Technical Reference Manual.
    unsafe fn get(&mut self) -> Self::Register;
}

/// Defines write access to MMIO and CPU registers.
///
/// Users may implement this trait for their own eligible types.
///
/// # Safety
///
/// This trait must only be implemented for register types that are
/// tagged [`Writable`][crate::perms::Writable] in accordance with the
/// Technical Reference Manual for the respective device.
pub unsafe trait RegisterWrite {
    /// The primitive type that represents the storage unit of
    /// the underlying register.
    type Register: Sealed + Copy;

    /// Writes the given raw `value` to the register.
    ///
    /// # A note on `&mut self`
    ///
    /// Many crates take `&self` when accessing registers. We would
    /// like to object with this position.
    ///
    /// As for write operations which mutate hardware state, this
    /// invariant should not be hidden by the public user API. See
    /// [`RegisterRead::get`] for further thoughts on the matter.
    ///
    /// # Safety
    ///
    /// Access permissions to the individual bit fields of the register
    /// are not enforced. The user needs to make sure to not accidentally
    /// write read-only or reserved fields.
    unsafe fn set(&mut self, value: Self::Register);
}

/// Defines mutual read and write access for MMIO and CPU registers.
///
/// Users may implement this trait for their own eligible types.
///
/// # Safety
///
/// This trait must only be implemented for register types that are
/// tagged [`Readable`][crate::perms::Readable] and
/// [`Writable`][crate::perms::Writable] in accordance with the
/// Technical Reference Manual for the respective device.
pub unsafe trait RegisterReadWrite {
    /// The primitive type that represents the storage unit of
    /// the underlying register.
    type Register: Sealed + Copy;
}
