use libc::{c_char, size_t};
use std::ffi::{CStr, CString};
use std::borrow::Cow;

use std::convert::From;
use std::fmt;
use std::ops::{Not, BitAnd, BitOr};

#[derive(Debug)]
#[repr(C)]
pub struct PyString {
    pub ptr: *const c_char,
    pub length: size_t,
}

impl PyString {
    pub unsafe fn from_ptr(ptr: *mut PyString) -> &'static PyString {
        &*ptr
    }
    pub unsafe fn into_string(&self) -> String {
        let c_str = CStr::from_ptr(self.ptr);
        String::from_utf8_lossy(c_str.to_bytes()).into_owned()
    }
    pub unsafe fn to_str(&self) -> Cow<str> {
        let c_str = CStr::from_ptr(self.ptr);
        String::from_utf8_lossy(c_str.to_bytes())
    }
    pub unsafe fn from_ptr_into_string(ptr: *mut PyString) -> String {
        let pystr: &PyString = &*ptr;
        let c_str = CStr::from_ptr(pystr.ptr);
        String::from_utf8_lossy(c_str.to_bytes()).into_owned()
    }
    pub fn as_ptr(self) -> *mut PyString {
        Box::into_raw(Box::new(self))
    }
}

impl fmt::Display for PyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{}", self.to_str()) }
    }
}

impl<'a> From<&'a str> for PyString {
    fn from(s: &'a str) -> PyString {
        PyString {
            ptr: s.as_ptr() as *const c_char,
            length: s.len(),
        }
    }
}

impl From<String> for PyString {
    fn from(s: String) -> PyString {
        let len = s.len();
        let c_str = unsafe { CString::from_vec_unchecked(s.into_bytes()).into_raw() };
        PyString {
            ptr: c_str as *const c_char,
            length: len,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PyBool {
    pub val: u8,
}

impl PyBool {
    pub unsafe fn from_ptr(ptr: *mut PyBool) -> &'static PyBool {
        &*ptr
    }
    pub unsafe fn from_ptr_into_bool(ptr: *mut PyBool) -> bool {
        let ptr: &PyBool = &*ptr;
        match ptr.val {
            0 => false,
            _ => true,
        }
    }
    pub fn to_bool(&self) -> bool {
        match self.val {
            0 => false,
            _ => true,
        }
    }
    pub fn as_ptr(self) -> *mut PyBool {
        Box::into_raw(Box::new(self))
    }
}

impl From<bool> for PyBool {
    fn from(b: bool) -> PyBool {
        let val = match b {
            true => 1,
            false => 0,
        };
        PyBool { val: val }
    }
}

impl<'a> From<&'a bool> for PyBool {
    fn from(b: &'a bool) -> PyBool {
        let val = match b {
            &true => 1,
            &false => 0,
        };
        PyBool { val: val }
    }
}

impl PartialEq<bool> for PyBool {
    fn eq(&self, other: &bool) -> bool {
        if self.val == 0 && *other == false {
            true
        } else if self.val == 1 && *other == true {
            true
        } else {
            false
        }
    }
}

impl<'a> PartialEq<bool> for &'a PyBool {
    fn eq(&self, other: &bool) -> bool {
        if self.val == 0 && *other == false {
            true
        } else if self.val == 1 && *other == true {
            true
        } else {
            false
        }
    }
}

impl Not for PyBool {
    type Output = bool;
    fn not(self) -> bool {
        match self.val {
            0 => false,
            _ => true,
        }
    }
}

impl BitAnd<bool> for PyBool {
    type Output = bool;
    fn bitand(self, rhs: bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val & rhs
    }
}

impl<'a> BitAnd<bool> for &'a PyBool {
    type Output = bool;
    fn bitand(self, rhs: bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val & rhs
    }
}

impl<'a> BitAnd<&'a bool> for PyBool {
    type Output = bool;
    fn bitand(self, rhs: &'a bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val & rhs
    }
}

impl<'a, 'b> BitAnd<&'a bool> for &'b PyBool {
    type Output = bool;
    fn bitand(self, rhs: &'a bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val & rhs
    }
}

impl BitOr<bool> for PyBool {
    type Output = bool;
    fn bitor(self, rhs: bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val | rhs
    }
}

impl<'a> BitOr<bool> for &'a PyBool {
    type Output = bool;
    fn bitor(self, rhs: bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val | rhs
    }
}

impl<'a> BitOr<&'a bool> for PyBool {
    type Output = bool;
    fn bitor(self, rhs: &'a bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val | rhs
    }
}

impl<'a, 'b> BitOr<&'a bool> for &'b PyBool {
    type Output = bool;
    fn bitor(self, rhs: &'a bool) -> bool {
        let val = match self.val {
            0 => false,
            _ => true,
        };
        val | rhs
    }
}

#[derive(Debug)]
pub struct PyTuple {
    pub elem: PyArg,
    pub idx: usize,
    pub next: Option<Box<PyTuple>>,
}

#[derive(Debug)]
pub enum PyArg {
    I64(i64),
    F32(f32),
    F64(f64),
    PyBool(PyBool),
    PyString(PyString),
}

impl<'a> PyTuple {
    fn get_element(&self, idx: usize) -> Result<&PyTuple, &str> {
        if idx == self.idx {
            Ok(&self)
        } else {
            match self.next {
                Some(ref e) => e.get_element(idx),
                None => Err("index out of range"),
            }
        }
    }
    fn len(&self) -> usize {
        match self.next {
            Some(ref e) => e.len(),
            None => self.idx + 1,
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn PyTuple_free(ptr: *mut PyTuple) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[macro_export]
macro_rules! pytuple {
    ( $( $elem:ident ),+ ) => {{
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
        Box::into_raw(Box::new(tuple.pop().unwrap()))
    }};
    ( $( $elem:expr ),+ ) => {{
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
        Box::into_raw(Box::new(tuple.pop().unwrap()))
    }};
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_len(ptr: *mut PyTuple) -> usize {
    let tuple = &*ptr;
    tuple.len()
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyInt(ptr: *mut PyTuple, index: usize) -> i64 {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::I64(val) => val,
        _ => panic!("expected i64, found other type"),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyBool(ptr: *mut PyTuple, index: usize) -> &'static PyBool {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::PyBool(ref val) => val,
        _ => panic!("expected PyBool, found other type"),
    }
}


#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyFloat(ptr: *mut PyTuple, index: usize) -> f32 {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::F32(val) => val,
        _ => panic!("expected f32, found other type"),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyDouble(ptr: *mut PyTuple, index: usize) -> f64 {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::F64(val) => val,
        _ => panic!("expected f64, found other type"),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyString(ptr: *mut PyTuple,
                                                 index: usize)
                                                 -> &'static PyString {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::PyString(ref val) => val,
        _ => panic!("expected PyString, found other type"),
    }
}

macro_rules! unpack_pytype {
    ( $ptr:ident: $pytype:ty ) => {{
        if $pytype == PyTuple {
        }
    }};
}
