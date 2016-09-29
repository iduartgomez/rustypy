use libc;
use std::ffi::{CString, CStr};
use libc::c_char;

use std::convert::From;
use std::fmt;
use std::ops::{Not, BitAnd, BitOr, Index, IndexMut};

pub struct PyList {
    members: Vec<PyArg>,
}

impl PyList {
    pub fn new() -> PyList {
        PyList { members: Vec::new() }
    }
    pub fn push(&mut self, value: PyArg) {
        self.members.push(value)
    }
    pub fn pop(&mut self) -> Option<PyArg> {
        self.members.pop()
    }
    pub fn remove(&mut self, index: usize) -> PyArg {
        self.members.remove(index)
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn PyList_free(ptr: *mut PyList) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

impl Index<usize> for PyList {
    type Output = PyArg;
    fn index(&self, index: usize) -> &PyArg {
        &(self.members[index])
    }
}

impl IndexMut<usize> for PyList {
    fn index_mut(&mut self, index: usize) -> &mut PyArg {
        &mut (self.members[index])
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyList_extractPyInt(ptr: *mut PyList, index: usize) -> i64 {
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

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyList_extractPyBool(ptr: *mut PyList, index: usize) -> PyBool {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::PyBool(val) => val,
        // _ => panic!("expected PyBool, found other type"),
        _ => abort_on_extraction_fail!(elem),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyList_extractPyFloat(ptr: *mut PyList, index: usize) -> f32 {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::F32(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyList_extractPyDouble(ptr: *mut PyList, index: usize) -> f64 {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::F64(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyList_extractPyString(ptr: *mut PyList, index: usize) -> PyString {
    let list = &mut *ptr;
    let elem = PyList::remove(list, index);
    match elem {
        PyArg::PyString(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}
