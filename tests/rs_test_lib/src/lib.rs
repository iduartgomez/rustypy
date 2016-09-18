#![allow(dead_code)]

extern crate libc;

use libc::{c_double, c_char};
use std::collections::HashMap;
use std::ffi::{CStr, CString};

// fn python_bind_fail() {
// println!("Hello from Rust!");
// }

#[no_mangle]
pub extern "C" fn python_bind_int(num: u32) -> u32 {
    num + 1
}

#[no_mangle]
pub extern "C" fn python_bind_ref_int(num: &mut u32) {
    *num += 1;
}

#[no_mangle]
pub extern "C" fn python_bind_str(pystr: *mut c_char) -> *mut c_char {
    let string = unsafe { CStr::from_ptr(pystr) };
    let mut modified = String::from_utf8_lossy(string.to_bytes()).into_owned();
    modified.push_str(" Added in Rust.");
    unsafe { CString::from_vec_unchecked(modified.into_bytes()).into_raw() }
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
pub extern "C" fn ython_bind_tuple(tup: (u32, u32)) -> (u32, u32) {
    return tup;
}

#[no_mangle]
pub extern "C" fn ython_bind_list2(_: Vec<(c_double, bool)>) -> Vec<String> {
    let returnval = vec![String::from("Rust")];
    returnval
}
