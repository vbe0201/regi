//! Implementations of register types to be used for Memory-Mapped I/O.
//!
//! The [`Register`] type is intended to be used for defining register
//! structs to generate APIs around, whereas [`RegisterWindow`]s can be
//! used for compile-time checked access to a register at runtime.

use core::marker::PhantomData;

use crate::{
    perms::{self, Permission},
    register::*,
    Int,
};

/// A [`Register`] type that allows read-only access to it.
pub type ReadOnly<I, R = ()> = Register<I, perms::ReadOnly, R>;
/// A [`Register`] type that allows write-only access to it.
pub type WriteOnly<I, R = ()> = Register<I, perms::WriteOnly, R>;
/// A [`Register`] type that allows read and write access to it.
pub type ReadWrite<I, R = ()> = Register<I, perms::ReadWrite, R>;

/// Representation of a Memory-Mapped I/O (MMIO) register.
///
/// This type should be used to define individual registers as part
/// of register structs.
///
/// `T` acts as the primitive integral type to read whereas `P` denotes
/// a [permission][crate::perms] value to enforce accidental misuse of
/// registers by interfacing through invalid operations.
///
/// Note that this type is only relevant for the very definition of a
/// register; users should not expect to obtain instances of this type.
/// See [`RegisterWindow`] for more information.
#[repr(transparent)]
pub struct Register<I: Int, P: Permission, R: RegisterMarker> {
    value: I,

    __perm: PhantomData<P>,
    __reg: PhantomData<R>,
}

// SAFETY: `Register` is repr(transparent) and we are therefore
// permitted to treat a pointer like a pointer to `T` itself.
impl<I: Int, P: Permission, R: RegisterMarker> Register<I, P, R> {
    #[inline]
    pub(super) unsafe fn get(self: *mut Self) -> I {
        self.read_volatile().value
    }

    #[inline]
    pub(super) unsafe fn set(self: *mut Self, value: I) {
        self.write_volatile(Register {
            value,
            __perm: PhantomData,
            __reg: PhantomData,
        });
    }
}

/// An access window to a [`Register`].
///
/// This is the type Rust code should interface with in order to access
/// an MMIO register.
///
/// Instances of this type are obtainable from generated APIs based on
/// register struct definitions. User code should **never** try to
/// construct its own instances of this type in any possible way.
pub struct RegisterWindow<'mmio, I: Int, P: Permission, R: RegisterMarker> {
    register: *mut Register<I, P, R>,

    __marker: PhantomData<&'mmio ()>,
}

// SAFETY: We can assume this type was constructed from a valid pointer
// or the mere existence of any objects would be UB.
impl<'mmio, I: Int, P: Permission, R: RegisterMarker> RegisterWindow<'mmio, I, P, R> {
    // Not part of the public API. Used by generated code.
    #[doc(hidden)]
    pub unsafe fn new(register: *mut Register<I, P, R>) -> Self {
        Self {
            register,

            __marker: PhantomData,
        }
    }
}

// SAFETY: Register has `Readable` permission.
unsafe impl<'mmio, I, P, R> RegisterRead for RegisterWindow<'mmio, I, P, R>
where
    I: Int,
    P: perms::Readable,
    R: RegisterMarker,
{
    type Register = I;
    type Marker = R;

    #[inline]
    unsafe fn get(&mut self) -> Self::Register {
        self.register.get()
    }
}

// SAFETY: Register has `Writable` permission.
unsafe impl<'mmio, I, P, R> RegisterWrite for RegisterWindow<'mmio, I, P, R>
where
    I: Int,
    P: perms::Writable,
    R: RegisterMarker,
{
    type Register = I;
    type Marker = R;

    #[inline]
    unsafe fn set(&mut self, value: Self::Register) {
        self.register.set(value)
    }
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_not_impl_all;

    use super::*;
    use crate::perms::ReadWrite;

    #[test]
    fn test_register_no_sync_send() {
        assert_not_impl_all!(RegisterWindow<u8, ReadWrite>: Sync, Send);
        assert_not_impl_all!(RegisterWindow<u16, ReadWrite>: Sync, Send);
        assert_not_impl_all!(RegisterWindow<u32, ReadWrite>: Sync, Send);
        assert_not_impl_all!(RegisterWindow<u64, ReadWrite>: Sync, Send);
    }

    #[test]
    fn test_register_no_clone_copy() {
        assert_not_impl_all!(RegisterWindow<u8, ReadWrite>: Clone, Copy);
        assert_not_impl_all!(RegisterWindow<u16, ReadWrite>: Clone, Copy);
        assert_not_impl_all!(RegisterWindow<u32, ReadWrite>: Clone, Copy);
        assert_not_impl_all!(RegisterWindow<u64, ReadWrite>: Clone, Copy);
    }
}
