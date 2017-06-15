#![allow(dead_code)]

extern crate libc;

#[macro_use]
extern crate rustypy;

pub mod nested;

use std::collections::HashMap;

use rustypy::{PyTuple, PyString, PyBool, PyArg, PyList, PyDict};

pub mod primitives {
    use super::*;

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
    let list = unsafe { Box::new(PyList::from_ptr(list)) };
    let converted = unpack_pylist!(list; PyList{PyString => String});
    assert_eq!(converted.len(), 3);
    for (i, e) in (&converted).iter().enumerate() {
        if i == 0 {
            assert_eq!(e, "Python");
        } else if i == 1 {
            assert_eq!(e, "in");
        } else if i == 2 {
            assert_eq!(e, "Rust");
        }
    }
    let content = vec!["Rust", "in", "Python"];
    let returnval = PyList::from(content);
    returnval.as_ptr()
}

#[no_mangle]
pub extern "C" fn other_prefix_dict(dict: *mut usize) -> *mut usize {
    let dict = unsafe { PyDict::<u64>::from_ptr(dict) };
    assert_eq!(dict.get(&0_u64), Some(&PyString::from("From")));
    assert_eq!(dict.get(&1_u64), Some(&PyString::from("Python")));
    let mut hm = HashMap::new();
    hm.insert(0_i64, PyArg::PyString(PyString::from("Back")));
    hm.insert(1_i64, PyArg::PyString(PyString::from("Rust")));
    PyDict::from(hm).as_ptr()
}
