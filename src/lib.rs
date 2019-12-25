
use std::marker::PhantomData;
use std::borrow::Cow;
use std::mem;
use std::ptr::NonNull;
use std::slice;
use std::ops;

#[derive(Debug)]
pub struct Calf<'a, T: Clone> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<&'a T>,
}

#[derive(Debug)]
pub struct ToMut<'a, 'b, T: Clone> {
    calf: &'b mut Calf<'a, T>,
    vec: Vec<T>,
}

impl<'a, T: Clone> Calf<'a, T> {
    pub fn owned(mut vec: Vec<T>) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };
        let len = vec.len();
        let cap = vec.capacity();
        mem::forget(vec);

        Calf { ptr, len, cap, _marker: PhantomData }
    }

    pub fn borrowed(slice: &'a [T]) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(slice.as_ptr() as *mut T) };
        let len = slice.len();

        Calf { ptr, len, cap: 0, _marker: PhantomData }
    }

    pub fn from_cow(cow: Cow<'a, [T]>) -> Self {
        match cow {
            Cow::Owned(v) => Self::owned(v),
            Cow::Borrowed(v) => Self::borrowed(v),
        }
    }

    pub fn into_cow(self) -> Cow<'a, [T]> {
        unsafe {
            if self.is_owned() {
                Cow::Owned(Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.cap))
            } else {
                Cow::Borrowed(slice::from_raw_parts(self.ptr.as_ptr(), self.len))
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }

    pub fn is_owned(&self) -> bool {
        self.cap != 0
    }

    pub fn into_owned(self) -> Vec<T> {
        self.into_cow().into_owned()
    }

    pub fn to_mut(&mut self) -> ToMut<'a, '_, T> {
        let this = mem::replace(self, Self::borrowed(&[]));

        ToMut { calf: self, vec: this.into_owned() }
    }
}

impl<'a, T: Clone> ops::Drop for Calf<'a, T> {
    fn drop(&mut self) {
        if self.is_owned() {
            unsafe {
                Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.cap);
            }
        }
    }
}

impl<'a, T: Clone> ops::Deref for Calf<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, 'b, T: Clone> ops::Drop for ToMut<'a, 'b, T> {
    fn drop(&mut self) {
        *self.calf = Calf::owned(mem::take(&mut self.vec));
    }
}

impl<'a, 'b, T: Clone> ops::Deref for ToMut<'a, 'b, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<'a, 'b, T: Clone> ops::DerefMut for ToMut<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
