#![allow(dead_code)]

extern crate libc;

#[macro_use]
extern crate rustypy;

use std::collections::HashMap;
use std::iter::FromIterator;
use rustypy::{PyTuple, PyString, PyBool, PyArg, PyList};

#[no_mangle]
pub extern "C" fn python_bind_int(num: u32) -> u32 {
    num + 1
}

#[no_mangle]
pub extern "C" fn python_bind_ref_int(num: &mut u32) {
    *num += 1;
}

#[no_mangle]
pub extern "C" fn python_bind_str(pystr: *mut PyString) -> *mut PyString {
    let mut string = unsafe { PyString::from_ptr_to_string(pystr) };
    assert_eq!(string, "From Python.");
    string.push_str(" Added in Rust.");

    PyString::from(string).as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_bool(ptr: *mut PyBool) -> *mut PyBool {
    let bool_t = unsafe { PyBool::from_ptr_into_bool(ptr) };
    assert!(bool_t);
    PyBool::from(false).as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_int_tuple(e1: i32, e2: i32) -> *mut PyTuple {
    pytuple!(PyArg::I32(e1), PyArg::I32(e2)).as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_str_tuple(e1: *mut PyString) -> *mut PyTuple {
    let s = PyString::from(unsafe { PyString::from_ptr_to_string(e1) });
    pytuple!(PyArg::PyString(s),
             PyArg::PyString(PyString::from("from Rust")))
        .as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_tuple_mixed(e1: i32,
                                          e2: *mut PyBool,
                                          e3: f32,
                                          e4: *mut PyString)
                                          -> *mut PyTuple {
    assert_eq!(unsafe { PyBool::from_ptr(e2) }, true);
    let s = PyString::from(unsafe { PyString::from_ptr_to_string(e4) });
    pytuple!(PyArg::I32(e1),
             PyArg::PyBool(PyBool::from(false)),
             PyArg::F32(e3),
             PyArg::PyString(s))
        .as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_list1(list: *mut PyList) -> *mut PyList {
    let mut list = unsafe { PyList::from_ptr(list) };
    let converted = unpack_pylist!(list; PyString => PyString);
    assert_eq!(converted.len(), 3);
    for (i, e) in (&converted).iter().enumerate() {
        if i == 0 {
            assert_eq!(e.to_string(), String::from("Python"));
        } else if i == 1 {
            assert_eq!(e.to_string(), String::from("in"));
        } else if i == 2 {
            assert_eq!(e.to_string(), String::from("Rust"));
        }
    }

    let content = vec![PyString::from(String::from("Rust")),
                       PyString::from(String::from("in")),
                       PyString::from(String::from("Python"))];
    let returnval = PyList::from_iter(content);
    returnval.as_ptr()
}

#[no_mangle]
pub extern "C" fn python_bind_list2(list: *mut PyList) -> *mut PyList {
    let mut list = unsafe { PyList::from_ptr(list) };
    let converted = unpack_pylist!(list; PyTuple => (I64, (F32, I64,),));
    assert_eq!(vec![(50i64, (1.0f32, 30i64)), (25i64, (0.5f32, 40i64))],
               converted);

    let v: Vec<PyTuple> = vec![pytuple!(PyArg::F64(0.5f64), PyArg::PyBool(PyBool::from(true))),
                               pytuple!(PyArg::F64(-0.5f64), PyArg::PyBool(PyBool::from(false)))];
    PyList::from_iter(v).as_ptr()
}

#[no_mangle]
pub extern "C" fn ython_bind_dict(dict: HashMap<&str, u32>) -> HashMap<&str, u32> {
    dict
}
