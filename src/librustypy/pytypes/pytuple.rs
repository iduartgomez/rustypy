//! An analog of a Python tuple, will accept an undefined number of any other supported types.
//!
//! You can construct it using the [pytuple!](../../macro.pytuple!.html) macro, ie:
//!
//! ```
//! # #[macro_use] extern crate rustypy;
//! # fn main(){
//! use rustypy::PyArg;
//! pytuple!(PyArg::I64(10), PyArg::F32(10.5));
//! # }
//! ```
//!
//! You must pass the variety of the argument using the PyArg enum.
//!
//! When extracting elements in Python with the FFI, elements are copied, not moved unless
//! possible (ie. content of inner containers may or may not be moved out),
//! and when free'd all the original elements are dropped.
//!
//! PyTuples behave exactly as Python tuples: they are immutable, but provide interior mutability.
//! For example, you can pop elements from an inner PyList, although the PyList cannot be moved
//! out of a PyTuple (without completely destructuring it).
//!
//! # Safety
//! PyTuple must be passed between Rust and Python as a raw pointer. You can get a
//! raw pointer using ```into_raw``` and convert from a raw pointer using the "static"
//! method ```PyDict::from_ptr``` which is unsafe as it requires dereferencing a raw pointer.
//!
//! ## Unpacking PyTuple from Python
//! Is recommended to use the [unpack_pytuple!](../../macro.unpack_pytuple!.html) macro in order
//! to convert a PyTuple to a Rust native type. Check the macro documentation for more info.

use std::iter::IntoIterator;
use std::mem;
use std::ops::Deref;

use crate::pytypes::PyArg;

/// An analog of a Python tuple, will accept an undefined number of other
/// [supported types](../../../rustypy/pytypes/enum.PyArg.html).
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone, Debug, PartialEq)]
pub struct PyTuple {
    pub(crate) elem: PyArg,
    pub(crate) idx: usize,
    pub(crate) next: Option<Box<PyTuple>>,
}

#[allow(clippy::len_without_is_empty)]
impl<'a> PyTuple {
    #[doc(hidden)]
    pub fn new(elem: PyArg, idx: usize, next: Option<Box<PyTuple>>) -> PyTuple {
        PyTuple { elem, idx, next }
    }

    #[doc(hidden)]
    pub fn set_next(&mut self, next: Option<Box<PyTuple>>) {
        self.next = next;
    }

    /// Get a PyTuple from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyTuple) -> PyTuple {
        *(Box::from_raw(ptr))
    }

    /// Get a mutable reference to an inner element of the tuple, takes as argument the position
    /// of the element and returns a Result.
    pub(crate) fn as_mut(&mut self, idx: usize) -> Result<&mut PyArg, &str> {
        if idx == self.idx {
            Ok(&mut self.elem)
        } else {
            match self.next {
                Some(ref mut e) => (**e).as_mut(idx),
                None => Err("PyTuple index out of range."),
            }
        }
    }

    #[doc(hidden)]
    pub fn replace_elem(&mut self, idx: usize) -> Result<PyArg, &str> {
        if idx == self.idx {
            let e = mem::replace(&mut self.elem, PyArg::None);
            Ok(e)
        } else {
            match self.next {
                Some(ref mut e) => (**e).replace_elem(idx),
                None => Err("PyTuple index out of range."),
            }
        }
    }

    /// Get a regular reference to an inner element of the tuple, takes as argument the position
    /// of the element and returns a Result.
    pub(crate) fn as_ref(&self, idx: usize) -> Result<&PyArg, &str> {
        if idx == self.idx {
            Ok(&self.elem)
        } else {
            match self.next {
                Some(ref e) => (**e).as_ref(idx),
                None => Err("PyTuple index out of range."),
            }
        }
    }

    fn push(&mut self, next: PyTuple) {
        self.next = Some(Box::new(next));
    }

    /// Returns the length of the tuple.
    pub fn len(&self) -> usize {
        match self.next {
            Some(ref e) => e.len(),
            None => self.idx + 1,
        }
    }

    /// Returns self as raw pointer. Use this method when returning a PyTuple to Python.
    pub fn into_raw(self) -> *mut PyTuple {
        Box::into_raw(Box::new(self))
    }
}

impl<'a> IntoIterator for &'a PyTuple {
    type Item = &'a PyArg;
    type IntoIter = ::std::vec::IntoIter<&'a PyArg>;
    fn into_iter(self) -> Self::IntoIter {
        let l = self.len();
        let mut iter = Vec::with_capacity(l);
        for i in 0..l {
            iter.push(self.as_ref(i).unwrap());
        }
        iter.into_iter()
    }
}

impl Deref for PyTuple {
    type Target = PyArg;

    fn deref(&self) -> &PyArg {
        &self.elem
    }
}

/// This macro allows the construction of [PyTuple](../rustypy/pytypes/pytuple/struct.PyTuple.html)
/// types.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use rustypy::PyArg;
/// pytuple!(PyArg::I64(10), PyArg::F32(10.5));
/// # }
/// ```
///
#[macro_export]
macro_rules! pytuple {
    ( $( $elem:ident ),+ ) => {{
        use rustypy::PyTuple;
        let mut cnt;
        let mut tuple = Vec::new();
        cnt = 0usize;
        $(
            let tuple_e = PyTuple::new(
                $elem,
                cnt,
                None,
            );
            tuple.push(tuple_e);
            cnt += 1;
        )*;
        if cnt == tuple.len() {}; // stub to remove warning...
        let t_len = tuple.len() - 1;
        for i in 1..(t_len + 1) {
            let idx = t_len - i;
            let last = tuple.pop().unwrap();
            let prev = tuple.get_mut(idx).unwrap();
            prev.set_next(Some(Box::new(last)));
        }
        tuple.pop().unwrap()
    }};
    ( $( $elem:expr ),+ ) => {{
        use rustypy::PyTuple;
        let mut cnt;
        let mut tuple = Vec::new();
        cnt = 0usize;
        $(
            let tuple_e = PyTuple::new(
                $elem,
                cnt,
                None,
            );
            tuple.push(tuple_e);
            cnt += 1;
        )*;
        let t_len = cnt - 1;
        for i in 1..cnt {
            let idx = t_len - i;
            let last = tuple.pop().unwrap();
            let prev = tuple.get_mut(idx).unwrap();
            prev.set_next(Some(Box::new(last)));
        }
        tuple.pop().unwrap()
    }};
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pytuple_new(idx: usize, elem: *mut PyArg) -> *mut PyTuple {
    let tuple = PyTuple {
        elem: *(Box::from_raw(elem)),
        idx,
        next: None,
    };
    tuple.into_raw()
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pytuple_push(next: *mut PyTuple, prev: &mut PyTuple) {
    let next: PyTuple = *(Box::from_raw(next));
    prev.push(next)
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pytuple_free(ptr: *mut PyTuple) {
    if ptr.is_null() {
        return;
    }

    Box::from_raw(ptr);
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pytuple_len(ptr: *mut PyTuple) -> usize {
    let tuple = &*ptr;
    tuple.len()
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pytuple_get_element(ptr: *mut PyTuple, index: usize) -> *mut PyArg {
    let tuple = &mut *ptr;
    let elem = &PyTuple::as_mut(tuple, index).unwrap();
    let copied: PyArg = (*elem).clone();
    Box::into_raw(Box::new(copied))
}
