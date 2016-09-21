use libc::{c_char, size_t};
use std::convert::From;
use std::ffi::{CStr, CString};
use std::fmt;
use std::borrow::Cow;

#[derive(Debug)]
#[repr(C)]
pub struct PyString {
    pub ptr: *const c_char,
    pub length: size_t,
}

impl PyString {
    pub unsafe fn into_string(&self) -> String {
        let c_str = CStr::from_ptr(self.ptr);
        String::from_utf8_lossy(c_str.to_bytes()).into_owned()
    }
    pub unsafe fn to_str(&self) -> Cow<str> {
        let c_str = CStr::from_ptr(self.ptr);
        String::from_utf8_lossy(c_str.to_bytes())
    }
}

impl fmt::Display for PyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{}", self.to_str()) }
    }
}

impl<'a> From<&'a str> for PyString {
    fn from(origin: &'a str) -> PyString {
        PyString {
            ptr: origin.as_ptr() as *const c_char,
            length: origin.len()
        }
    }
}

impl From<String> for PyString {
    fn from(origin: String) -> PyString {
        let len = origin.len();
        let c_str = unsafe { CString::from_vec_unchecked(origin.into_bytes()).into_raw() };
        PyString {
            ptr: c_str as *const c_char,
            length:len
        }
    }
}

#[repr(C)]
pub struct PyBool {
    pub val: u8,
}

impl<'a> From<&'a bool> for PyBool {
    fn from(origin: &'a bool) -> PyBool {
        let val = match origin {
            &true => 1,
            &false => 0,
        };
        PyBool {val: val}
    }
}

#[derive(Debug)]
pub struct PyTuple {
    pub array: Vec<size_t>,
}

#[macro_export]
macro_rules! pytuple {
    ( $( $elem:ident ),+ ) => {{
        let mut vec: Vec<size_t> = Vec::new();
        $(
            let elem_ptr = Box::into_raw(Box::new($elem));
            vec.push(elem_ptr as size_t);
        )*;
        let tuple = PyTuple {
            array: vec,
        };
        Box::into_raw(Box::new(tuple))
    }};
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_len(ptr: *const PyTuple) -> u32 {
    let tuple = &*ptr;
    tuple.array.len() as u32
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyInt(ptr: *const PyTuple, elem: u32) -> i64 {
    let tuple = &*ptr;
    let int = tuple.array[elem as usize] as *const i64;
    *int
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyFloat(ptr: *const PyTuple, elem: u32) -> f32 {
    let tuple = &*ptr;
    let float = tuple.array[elem as usize] as *const f32;
    *float
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyDouble(ptr: *const PyTuple, elem: u32) -> f64 {
    let tuple = &*ptr;
    let float = tuple.array[elem as usize] as *const f64;
    *float
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyBool(ptr: *const PyTuple, elem: u32) -> *mut PyBool {
    let tuple = &*ptr;
    tuple.array[elem as usize] as *mut PyBool
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyString(ptr: *const PyTuple, elem: u32) -> *mut PyString {
    let tuple = &*ptr;
    tuple.array[elem as usize] as *mut PyString
}

macro_rules! unpack_pytype {
    ( $ptr:ident: $pytype:ty ) => {{
        if $pytype == PyTuple {

        }
    }};
}
