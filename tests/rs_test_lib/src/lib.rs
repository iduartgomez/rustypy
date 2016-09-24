#![allow(dead_code)]

extern crate libc;

#[macro_use]
extern crate rustypy;

use std::collections::HashMap;
use rustypy::{PyTuple, PyString, PyBool, PyArg};

#[no_mangle]
pub extern "C" fn python_bind_int(num: u32) -> u32 {
    num + 1
}

#[no_mangle]
pub extern "C" fn python_bind_ref_int(num: &mut u32) {
    *num += 1;
}

#[no_mangle]
pub extern "C" fn python_bind_str_by_ref(pystr: *mut PyString) -> *mut PyString {
    let mut string: String = unsafe { PyString::from_ptr_into_string(pystr) };
    string.push_str(" Added in Rust.");
    PyString::from(string).as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_str(pystr: *mut PyString) -> PyString {
    let mut string: String = unsafe { PyString::from_ptr_into_string(pystr) };
    string.push_str(" Added in Rust.");
    PyString::from(string)
}

#[no_mangle]
pub extern "C" fn python_bind_bool(ptr: *mut PyBool) -> PyBool {
    let bool_t = unsafe { PyBool::from_ptr_into_bool(ptr) };
    assert!(bool_t);
    PyBool::from(false)
}

#[no_mangle]
pub extern "C" fn python_bind_tuple(e1: i32, e2: i32) -> *mut PyTuple {
    pytuple!(PyArg::I64(e1 as i64), PyArg::I64(e2 as i64))
}

#[no_mangle]
pub extern "C" fn python_bind_str_tuple(e1: *mut PyString) -> *mut PyTuple {
    let s = PyString::from(unsafe { PyString::from_ptr_into_string(e1) });
    pytuple!(PyArg::PyString(s),
             PyArg::PyString(PyString::from("from Rust")))
}

#[no_mangle]
pub extern "C" fn python_bind_tuple_mixed(e1: i32,
                                          e2: *mut PyBool,
                                          e3: f32,
                                          e4: *mut PyString)
                                          -> *mut PyTuple {
    assert_eq!(unsafe { PyBool::from_ptr(e2) }, true);
    let s = PyString::from(unsafe { PyString::from_ptr_into_string(e4) });
    pytuple!(PyArg::I64(e1 as i64),
             PyArg::PyBool(PyBool::from(false)),
             PyArg::F32(e3),
             PyArg::PyString(s))
}

#[no_mangle]
pub extern "C" fn ython_bind_list1(_: Vec<bool>) -> Vec<String> {
    let returnval = vec![String::from("Rust")];
    returnval
}

#[no_mangle]
pub extern "C" fn ython_bind_dict(dict: HashMap<&str, u32>) -> HashMap<&str, u32> {
    dict
}

#[no_mangle]
pub extern "C" fn ython_bind_list2(_: Vec<(f64, bool)>) -> Vec<String> {
    let returnval = vec![String::from("Rust")];
    returnval
}
