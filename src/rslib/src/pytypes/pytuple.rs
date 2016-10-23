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
//! raw pointer using ```as_ptr``` and convert from a raw pointer using the "static"
//! method ```PyDict::from_ptr``` which is unsafe as it requires dereferencing a raw pointer.
//!
//! ## Unpacking PyTuple from Python
//! Is recommended to use the [unpack_pytuple!](../../macro.unpack_pytuple!.html) macro in order
//! to convert a PyTuple to a Rust native type. Check the macro documentation for more info.

use std::iter::IntoIterator;
use std::ops::Deref;
use std::mem;

use pytypes::PyArg;

/// An analog of a Python tuple, will accept an undefined number of other
/// [supported types](../../../rustypy/pytypes/enum.PyArg.html).
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone, Debug, PartialEq)]
pub struct PyTuple {
    pub elem: PyArg,
    pub idx: usize,
    pub next: Option<Box<PyTuple>>,
}

impl<'a> PyTuple {
    /// Get a PyTuple from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyTuple) -> PyTuple {
        *(Box::from_raw(ptr))
    }
    /// Get a mutable reference to an inner element of the tuple, takes as argument the position
    /// of the element and returns a Result.
    pub fn as_mut(&mut self, idx: usize) -> Result<&mut PyArg, &str> {
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
    pub fn as_ref(&self, idx: usize) -> Result<&PyArg, &str> {
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
    pub fn len(&self) -> usize {
        match self.next {
            Some(ref e) => e.len(),
            None => self.idx + 1,
        }
    }
    /// Returns self as raw pointer. Use this method when returning a PyTuple to Python.
    pub fn as_ptr(self) -> *mut PyTuple {
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
            let tuple_e = PyTuple {
                elem: $elem,
                idx: cnt,
                next: None,
            };
            tuple.push(tuple_e);
            cnt += 1;
        )*;
        if cnt == tuple.len() {}; // stub to remove warning...
        let t_len = tuple.len() - 1;
        for i in 1..(t_len + 1) {
            let idx = t_len - i;
            let last = tuple.pop().unwrap();
            let prev = tuple.get_mut(idx).unwrap();
            prev.next = Some(Box::new(last));
        }
        tuple.pop().unwrap()
    }};
    ( $( $elem:expr ),+ ) => {{
        use rustypy::PyTuple;
        let mut cnt;
        let mut tuple = Vec::new();
        cnt = 0usize;
        $(
            let tuple_e = PyTuple {
                elem: $elem,
                idx: cnt,
                next: None,
            };
            tuple.push(tuple_e);
            cnt += 1;
        )*;
        if cnt == 0 {}; // stub to remove warning...
        let t_len = tuple.len() - 1;
        for i in 1..(t_len + 1) {
            let idx = t_len - i;
            let last = tuple.pop().unwrap();
            let prev = tuple.get_mut(idx).unwrap();
            prev.next = Some(Box::new(last));
        }
        tuple.pop().unwrap()
    }};
}

/// Iterates over a a PyTuple and returns a corresponding Rust tuple.
///
/// Content of tuples cannot be moved out when destructured, so all inner data is copied
/// except when avoidable (ie. the content of inner container types may or may not be moved),
/// but when unpacked is safer to assume the PyTuple as consumed
/// (that's the intetion of unpacking).
///
/// Inner containers (ie. `PyList<PyArg(T)>`) are converted to the respective Rust analog
/// (ie. `Vec<T>`) and require valid syntax for their respective unpack macro (ie.
/// [unpack_pytuple!](../rustypy/macro.unpack_pylist!.html)).
///
/// # Examples
///
/// Unpack a PyTuple which contains a two PyDict types with PyString keys
/// and values of PyList<i64>:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use rustypy::{PyDict, PyList, PyTuple, PyArg, PyString};
/// # use std::collections::HashMap;
/// # let mut hm = HashMap::new();
/// # hm.insert(PyString::from("one"), vec![0_i32, 1, 2]);
/// # hm.insert(PyString::from("two"), vec![3_i32, 2, 1]);
/// # let mut pytuple = pytuple!(PyArg::PyDict(PyDict::from(hm.clone()).as_ptr()),
/// #                            PyArg::PyDict(PyDict::from(hm.clone()).as_ptr())).as_ptr();
/// // tuple from Python: ({"one": [0, 1, 3], "two": [3, 2, 1]})
/// let mut pytuple = unsafe { PyTuple::from_ptr(pytuple) };
/// let unpacked = unpack_pytuple!(pytuple; ({PyDict{(PyString, PyList{I32 => i64})}},
///                                          {PyDict{(PyString, PyList{I32 => i64})}},));
/// # }
/// ```
///
#[macro_export]
macro_rules! unpack_pytuple {
    ($t:ident; ($($p:tt,)+) ) => {{
        use rustypy::PyArg;
        let mut cnt = 0;
        ($(
            unpack_pytuple!($t; cnt; elem: $p)
        ,)*)
    }};
    ($t:ident; $i:ident; elem: ($($p:tt,)+))  => {{
        let e = $t.replace_elem($i).unwrap();
        match e {
            PyArg::PyTuple(val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                let mut cnt = 0;
                let val = *(val); // move out of box
                ($(
                    unpack_pytuple!(val; cnt; elem: $p)
                ,)*)
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a PyTuple inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: {PyDict{$u:tt}}) => {{
        let e = $t.as_mut($i).unwrap();
        match e {
            &mut PyArg::PyDict(val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                unpack_pydict!(val; PyDict{$u})
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a PyDict inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: {PyList{$($u:tt)*}}) => {{
        let e = $t.as_mut($i).unwrap();
        match e {
            &mut PyArg::PyList(ref mut val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                unpack_pylist!( FROM_TUPLE: val; PyList{$($u)*})
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a PyList inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: PyBool) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::PyBool(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.to_bool()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a PyBool inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: PyString) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::PyString(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.to_string()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a PyString inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I64) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::I64(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a i64 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I32) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::I32(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a i32 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I16) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::I16(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a i16 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I8) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &mut PyArg::I8(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a i8 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: U32) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::U32(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a u32 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: U16) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::U16(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a u16 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: U8) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::U8(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a u8 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: F32) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::F32(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a f32 inside a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: F64) => {{
        let e = $t.as_ref($i).unwrap();
        match e {
            &PyArg::F64(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while extracting a f64 inside a PyTuple"),
        }
    }};
}

#[no_mangle]
pub unsafe extern "C" fn pytuple_new(idx: usize, elem: *mut PyArg) -> *mut PyTuple {
    let tuple = PyTuple {
        elem: *(Box::from_raw(elem)),
        idx: idx,
        next: None,
    };
    tuple.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn pytuple_push(next: *mut PyTuple, prev: &mut PyTuple) {
    let next: PyTuple = *(Box::from_raw(next));
    prev.push(next)
}

#[no_mangle]
pub extern "C" fn pytuple_free(ptr: *mut PyTuple) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn pytuple_len(ptr: *mut PyTuple) -> usize {
    let tuple = unsafe { &*ptr };
    tuple.len()
}

#[no_mangle]
pub unsafe extern "C" fn pytuple_get_element(ptr: *mut PyTuple, index: usize) -> *mut PyArg {
    let tuple = &mut *ptr;
    let ref elem = PyTuple::as_mut(tuple, index).unwrap();
    let copied: PyArg = (*elem).clone();
    Box::into_raw(Box::new(copied))
}
