//! An analog of a Python list which contains elements of a single type, will accept an
//! undefined number of one (and just one) of any other supported type (including other
//! PyLists).
//!
//! PyList can be constructed from other iterable types as long as the inner type is
//! supported (a copy will be performed in case is necessary).
//!
//! ```
//! # use rustypy::PyList;
//! # use std::iter::FromIterator;
//! PyList::from_iter(vec![1u32; 3]);
//! PyList::from(vec![1u32; 3]);
//! ```
//!
//! You can also use the typical vector interfaces (push, pop, remove, etc.) as long as the
//! type is supported (check [PyArg](../rustypy/pytypes/enum.PyArg.html) variants). PyList
//! types and their content can also be implicitly converted through an special iterator type.
//!
//! ```
//! # use rustypy::PyList;
//! let mut l = PyList::new();
//! for e in vec![0u32, 1, 3] {
//!     l.push(e);
//! }
//!
//! let mut iter = PyList::speciallized_iter::<u32>(l);
//! assert_eq!(iter.collect::<Vec<u32>>(), vec![0u32, 1, 3])
//! ```
//!
//! When extracting in Python with the FFI, elements are moved whenever is possible, not copied
//! and when free'd all the original elements are dropped.
//!
//! # Safety
//! PyList must be passed between Rust and Python as a raw pointer. You can get a raw pointer
//! using ```into_raw``` and convert from a raw pointer using the "static"
//! method ```PyList::from_ptr``` which is unsafe as it requires dereferencing a raw pointer.
//!
//! For convinience there are some methods to perform conversions to Vec<T> from PyList<PyArg>,
//! while none of those are unsafe per se, they require providing the expected PyArg enum variant.
//! In case the expected variant is wrong, the process will abort and exit as it's not possible
//! to handle errors acrosss the FFI boundary.
//!
//! ## Unpacking PyList from Python
//! Is recommended to use the [unpack_pylist!](../../macro.unpack_pylist!.html) macro in order
//! to convert a PyList to a Rust native type. Check the macro documentation for more info.

use super::PyArg;

use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// An analog of a Python list which contains an undefined number of elements of
/// a single kind, of any [supported type](../../../rustypy/pytypes/enum.PyArg.html).
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PyList {
    _inner: Vec<PyArg>,
}

impl PyList {
    /// Constructs a new, empty ```PyList<T>```.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    pub fn new() -> PyList {
        PyList { _inner: Vec::new() }
    }

    /// Removes and returns the element at position ```index``` within the vector,
    /// shifting all elements after it to the left.
    pub fn remove<T>(&mut self, index: usize) -> T
    where
        T: From<PyArg>,
    {
        T::from(self._inner.remove(index))
    }

    /// Removes the last element from a vector and returns it, or ```None``` if it is empty.
    pub fn pop<T>(&mut self) -> Option<T>
    where
        T: From<PyArg>,
    {
        if let Some(val) = self._inner.pop() {
            Some(T::from(val))
        } else {
            None
        }
    }

    /// Returns the number of elements in the PyList.
    pub fn len(&self) -> usize {
        self._inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self._inner.is_empty()
    }

    /// Appends an element to the back of a collection.
    ///
    /// ##Panics
    ///
    /// Panics if the number of elements in the vector overflows a usize.
    pub fn push<T>(&mut self, a: T)
    where
        PyArg: From<T>,
    {
        self._inner.push(PyArg::from(a))
    }

    /// Get a PyList from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyList) -> PyList {
        *(Box::from_raw(ptr))
    }

    /// Return a PyList as a raw pointer.
    pub fn into_raw(self) -> *mut PyList {
        Box::into_raw(Box::new(self))
    }

    /// Consume self and turn it into an iterator.
    pub fn speciallized_iter<T: From<PyArg>>(self) -> IntoIter<T> {
        IntoIter {
            inner: self._inner.into_iter(),
            target_t: PhantomData,
        }
    }
}

impl<T> FromIterator<T> for PyList
where
    PyArg: From<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.push(e);
        }
        c
    }
}

pub struct IntoIter<T> {
    target_t: PhantomData<T>,
    inner: ::std::vec::IntoIter<PyArg>,
}

impl<T> Iterator for IntoIter<T>
where
    T: From<PyArg>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.inner.next() {
            Some(val) => Some(<T>::from(val)),
            None => None,
        }
    }
    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
    {
        self.inner.map(<T>::from).collect::<B>()
    }
}

impl<T> Into<Vec<T>> for PyList
where
    PyArg: Into<T>,
{
    fn into(mut self) -> Vec<T> {
        self._inner.drain(..).map(PyArg::into).collect()
    }
}

impl<T> From<Vec<T>> for PyList
where
    PyArg: From<T>,
{
    fn from(mut v: Vec<T>) -> PyList {
        PyList {
            _inner: v.drain(..).map(PyArg::from).collect(),
        }
    }
}

impl Index<usize> for PyList {
    type Output = PyArg;
    fn index(&self, index: usize) -> &PyArg {
        &(self._inner[index])
    }
}

impl<'a> IndexMut<usize> for PyList {
    fn index_mut(&mut self, index: usize) -> &mut PyArg {
        &mut (self._inner[index])
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pylist_new(len: usize) -> *mut PyList {
    let list = PyList {
        _inner: Vec::with_capacity(len),
    };
    list.into_raw()
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pylist_push(list: &mut PyList, e: *mut PyArg) {
    list.push(*(Box::from_raw(e)));
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pylist_len(list: &mut PyList) -> usize {
    list.len()
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pylist_free(ptr: *mut PyList) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pylist_get_element(ptr: *mut PyList, index: usize) -> *mut PyArg {
    let list = &mut *ptr;
    Box::into_raw(Box::new(PyList::remove(list, index)))
}
