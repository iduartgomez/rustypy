#![allow(dead_code)]

extern crate libc;

#[macro_use]
extern crate rustypy;

use libc::size_t;
use std::collections::HashMap;
use rustypy::{PyTuple, PyString, PyBool};

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
pub extern "C" fn python_bind_tuple(e1: i32, e2: i32) -> *mut PyTuple {
    pytuple!(e1, e2)
}

#[no_mangle]
pub extern "C" fn python_bind_tuple_mixed(e1: i32,
                                          e2: *mut PyBool,
                                          e3: f32,
                                          e4: *mut PyString)
                                          -> *mut PyTuple {
    pytuple!(e1, e2, e3, e4)
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
