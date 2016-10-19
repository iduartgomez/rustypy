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
//! When extracting elements in Python with the FFI, elements are copied, not moved,
//! and when free'd all the original elements are dropped.
//!
//! ## Unpacking PyTuple from Python
//! Is recommended to use the [unpack_pytuple!](../../macro.unpack_pytuple!.html) macro in order
//! to convert a PyTuple to a Rust native type. Check the macro documentation for more info.

use std::iter::IntoIterator;
use std::ops::Deref;

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
    fn get_element(&self, idx: usize) -> Result<&PyTuple, &str> {
        if idx == self.idx {
            Ok(self)
        } else {
            match self.next {
                Some(ref e) => e.get_element(idx),
                None => Err("PyTuple index out of range."),
            }
        }
    }
    pub fn get_inner_ref(&self, idx: usize) -> Result<&PyArg, &str> {
        if idx == self.idx {
            Ok(&self.elem)
        } else {
            match self.next {
                Some(ref e) => e.get_inner_ref(idx),
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
            iter.push(self.get_inner_ref(i).unwrap());
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
/// PyTuple types are inmmutable, all inner types contents are copied when destructured
/// (this includes inner container types like PyList or nested PyTuples).
/// Inner `PyList<PyArg(T)>` are converted to the respective `Vec<T>` and requires
/// valid syntax for [unpack_pytuple!](../rustypy/macro.unpack_pylist!.html).
///
/// # Examples
///
/// ```
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
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::PyTuple(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                let pytuple = val.clone();
                let mut cnt = 0;
                ($(
                    unpack_pytuple!(pytuple; cnt; elem: $p)
                ,)*)
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: {PyList{$($u:tt)*}}) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::PyList(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                let copied = (*val).clone();
                unpack_pylist!(copied; PyList{$($u)*})
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: PyBool) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::PyBool(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.to_bool()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: PyString) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::PyString(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.to_string()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I64) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::I64(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I32) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::I32(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I16) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::I16(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: I8) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::I8(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: U32) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::U32(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: U16) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::U16(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: U8) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::U8(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: F32) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::F32(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
        }
    }};
    ($t:ident; $i:ident; elem: F64) => {{
        let e = $t.get_inner_ref($i).unwrap();
        match e {
            &PyArg::F64(ref val) => {
                $i += 1;
                if $i == 0 {}; // stub to remove warning...
                val.clone()
            },
            _ => _rustypy_abort_xtract_fail!("failed while unpacking a PyTuple"),
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
    let tuple = &*ptr;
    let ref elem = PyTuple::get_element(tuple, index).unwrap().elem;
    let copied: PyArg = elem.clone();
    Box::into_raw(Box::new(copied))
}
