// #![no_std]

use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use core::ptr;
use std::ptr::NonNull;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq)]
pub struct PayloadPointer<T: ?Sized + Pointee> {
    ptr: NonNull<()>,
    meta: <T as Pointee>::Metadata,
    _marker: PhantomData<*const T>,
}

pub trait Pointee {
    type Metadata: Debug + Copy + Send + Sync + Ord + Hash + Unpin;
}

impl<T> Pointee for [T] {
    type Metadata = usize;
}

impl Pointee for str {
    type Metadata = usize;
}

impl<T: ?Sized + Pointee> PayloadPointer<T> {
    /// Returns the metadata of the pointee.
    pub const fn metadata_of(self) -> T::Metadata {
        self.meta
    }

    pub fn addr(self) -> usize {
        self.ptr.addr().into()
    }

    pub fn as_ptr(self) -> NonNull<T>
    where
        T: Sized,
    {
        self.ptr.cast()
    }

    /// # Safety
    /// equivalent to core::ptr::read on the pointee.
    pub unsafe fn deref<P: Pointee + Sized>(pp: PayloadPointer<P>) -> P {
        unsafe { core::ptr::read(pp.ptr.as_ptr().cast()) }
    }

    pub const fn from_raw_parts(ptr: NonNull<()>, meta: <T as Pointee>::Metadata) -> PayloadPointer<T> {
        PayloadPointer {
            ptr,
            meta,
            _marker: PhantomData,
        }
    }

    pub const fn into_raw_parts(self) -> (NonNull<T>, <T as Pointee>::Metadata)
    where
        T: Sized,
    {
        (self.ptr.cast(), self.meta)
    }
}

impl<T> PayloadPointer<[T]> {
    pub const fn to_raw_slice(self) -> NonNull<[T]>
    where
        T: Sized,
    {
        let sc = ptr::slice_from_raw_parts_mut(self.ptr.as_ptr().cast(), self.meta);
        unsafe { NonNull::new_unchecked(sc) }
    }
}

impl PayloadPointer<str> {
    pub const fn to_raw_str(self) -> *const str {
        ptr::slice_from_raw_parts(self.ptr.as_ptr().cast::<u8>(), self.meta) as *const str
    }
}

pub unsafe trait GetRawPtr<AddrSource: ?Sized>
where
    Self: Pointee,
{
    /// Address comes from `&AddrSource`, metadata comes from `meta`. using the metadata field of `self` is prohibited.
    /// This means you can't call `slice.len()` on &[T] or similar.
    fn get_raw_const_ptr_from_ref(addr: &AddrSource, meta: <Self as Pointee>::Metadata) -> PayloadPointer<Self> {
        let nn = unsafe { NonNull::new_unchecked(addr as *const AddrSource as *mut ()) };
        PayloadPointer::from_raw_parts(nn, meta)
    }
    /// Address comes from `&mut AddrSource`, metadata comes from `meta`. using the metadata field of `self` is prohibited.
    /// This means you can't call `slice.len()` on &[T] or similar.
    fn get_raw_mut_ptr_from_ref(addr: &mut AddrSource, meta: <Self as Pointee>::Metadata) -> PayloadPointer<Self> {
        let nn = unsafe { NonNull::new_unchecked(addr as *mut AddrSource as *mut ()) };
        PayloadPointer::from_raw_parts(nn, meta)
    }
}

unsafe impl<T> GetRawPtr<[T]> for [T] {}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[derive(Clone, Copy)]
pub struct RawSlice2D<T> {
    _marker: PhantomData<T>,
}

unsafe impl<T> GetRawPtr<[T]> for RawSlice2D<T> {}
impl<T> Pointee for RawSlice2D<T> {
    // lenx, leny. slice2d[meta.0 - 1][meta.1 - 1] always succeeds.
    type Metadata = (usize, usize);
}
#[test]
fn test_2d_slice() {
    let data = [
        [0, 1, 2], //
        [3, 4, 5], //
        [6, 7, 8],
    ];

    let slice2d = RawSlice2D::get_raw_const_ptr_from_ref(&data, (3, 3));
    println!(
        "Addr: {:p}\nHorizontal Len: {}, Vertical Len: {}\nMOST IMPORTANTLY, size_of::<PayloadPointer<RawSlice2D<i32>>>(): {}",
        slice2d.as_ptr().as_ptr(),
        slice2d.metadata_of().0,
        slice2d.metadata_of().1,
        size_of::<PayloadPointer<RawSlice2D<i32>>>()
    );
}
