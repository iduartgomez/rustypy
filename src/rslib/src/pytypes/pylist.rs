use libc;
use pytypes::{PyArg, PyBool, PyString};

use std::ops::{Index, IndexMut};
use std::iter::{FromIterator, IntoIterator};

#[derive(Debug)]
pub struct PyList {
    members: Vec<PyArg>,
}

impl PyList {
    pub fn new() -> PyList {
        PyList { members: Vec::new() }
    }
    pub fn remove(&mut self, index: usize) -> PyArg {
        self.members.remove(index)
    }
    pub fn pop(&mut self) -> Option<PyArg> {
        self.members.pop()
    }
    pub fn len(&self) -> usize {
        self.members.len()
    }
    pub unsafe fn from_ptr(ptr: *mut PyList) -> PyList {
        *(Box::from_raw(ptr))
    }
    pub fn as_ptr(self) -> *mut PyList {
        Box::into_raw(Box::new(self))
    }
}

/// Consumes a PyList<PyArg(T)> and returns a Vec<T> from it, no copies.
#[macro_export]
macro_rules! pylist_to_vec {
    ( $pylist:ident; $p:path => $type_:ty ) => {{
        use rustypy::PyArg;
        trait PyListPop {
            type Target;
            fn pop_t(&mut self) -> Option<Self::Target>;
        }

        impl PyListPop for PyList {
            type Target = $type_;
            fn pop_t(&mut self) -> Option<Self::Target> {
                match self.pop() {
                    Some($p(val)) => Some(val),
                    Some(_) => panic!("rustypy error: unexpected type on PyList"),
                    None => None
                }
            }
        }
        let mut list: Vec<$type_> = Vec::new();
        for _ in 0..$pylist.len() {
            match $pylist.pop_t() {
                Some(v) => list.push(v),
                None => {}
            }
        };
        list
    }}
}



impl IntoIterator for PyList {
    type Item = PyArg;
    type IntoIter = ::std::vec::IntoIter<PyArg>;
    fn into_iter(self) -> Self::IntoIter {
        self.members.into_iter()
    }
}

trait PyListPush<T> {
    fn push(&mut self, e: T);
}

impl PyListPush<i64> for PyList {
    fn push(&mut self, e: i64) {
        self.members.push(PyArg::I64(e));
    }
}

impl FromIterator<i64> for PyList {
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.members.push(PyArg::I64(e))
        }
        c
    }
}

impl PyListPush<f32> for PyList {
    fn push(&mut self, e: f32) {
        self.members.push(PyArg::F32(e));
    }
}

impl FromIterator<f32> for PyList {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.members.push(PyArg::F32(e))
        }
        c
    }
}

impl PyListPush<f64> for PyList {
    fn push(&mut self, e: f64) {
        self.members.push(PyArg::F64(e));
    }
}

impl FromIterator<f64> for PyList {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.members.push(PyArg::F64(e))
        }
        c
    }
}

impl PyListPush<PyString> for PyList {
    fn push(&mut self, e: PyString) {
        self.members.push(PyArg::PyString(e));
    }
}

impl FromIterator<PyString> for PyList {
    fn from_iter<I: IntoIterator<Item = PyString>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.members.push(PyArg::PyString(e))
        }
        c
    }
}

impl PyListPush<PyBool> for PyList {
    fn push(&mut self, e: PyBool) {
        self.members.push(PyArg::PyBool(e));
    }
}

impl FromIterator<PyBool> for PyList {
    fn from_iter<I: IntoIterator<Item = PyBool>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.members.push(PyArg::PyBool(e))
        }
        c
    }
}

impl PyListPush<PyArg> for PyList {
    fn push(&mut self, e: PyArg) {
        self.members.push(e);
    }
}

impl FromIterator<PyArg> for PyList {
    fn from_iter<I: IntoIterator<Item = PyArg>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.members.push(e)
        }
        c
    }
}

impl Index<usize> for PyList {
    type Output = PyArg;
    fn index(&self, index: usize) -> &PyArg {
        &(self.members[index])
    }
}

impl<'a> IndexMut<usize> for PyList {
    fn index_mut(&mut self, index: usize) -> &mut PyArg {
        &mut (self.members[index])
    }
}

#[no_mangle]
pub unsafe extern "C" fn pylist_new(len: usize) -> *mut PyList {
    let list = PyList { members: Vec::with_capacity(len) };
    list.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn pylist_push(list: &mut PyList, e: *mut PyArg) {
    list.push(*(Box::from_raw(e)));
}

#[no_mangle]
pub unsafe extern "C" fn pylist_len(list: &mut PyList) -> usize {
    list.len()
}

#[no_mangle]
pub extern "C" fn pylist_free(ptr: *mut PyList) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}


#[no_mangle]
pub unsafe extern "C" fn pylist_extract_pyint(ptr: *mut PyList, index: usize) -> i64 {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::I64(val) => val,
        PyArg::I32(val) => val as i64,
        PyArg::I16(val) => val as i64,
        PyArg::I8(val) => val as i64,
        PyArg::U32(val) => val as i64,
        PyArg::U16(val) => val as i64,
        PyArg::U8(val) => val as i64,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pylist_extract_pybool(ptr: *mut PyList, index: usize) -> PyBool {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::PyBool(val) => val,
        // _ => panic!("expected PyBool, found other type"),
        _ => abort_on_extraction_fail!(elem),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pylist_extract_pyfloat(ptr: *mut PyList, index: usize) -> f32 {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::F32(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pylist_extract_pydouble(ptr: *mut PyList, index: usize) -> f64 {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::F64(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pylist_extract_pystring(ptr: *mut PyList, index: usize) -> *mut PyString {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::PyString(val) => val.as_ptr(),
        _ => abort_on_extraction_fail!(elem),
    }
}
