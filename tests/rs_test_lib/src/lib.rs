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

/*#[no_mangle]
pub extern "C" fn python_bind_str(pystr: *mut PyString) -> PyString {
    let mut s: String = pystr.into_string();
    s.push_str(" Added in Rust.");
    PyString::from(s)
}*/

#[no_mangle]
pub extern "C" fn python_bind_tuple(e1: u32,
                                    e2: f32,
                                    e3: PyBool,
                                    e4: *mut PyString)
                                    -> *mut PyTuple {
    pytuple!(e1, e2, e3, e4)
}

#[no_mangle]
pub extern "C" fn ython_bind_two_ints(num1: u32, num2: u32) -> (u32, u32) {
    return (num1, num2);
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
