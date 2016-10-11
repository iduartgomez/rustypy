//! An analog of a Python String.
//!
//! To return to Python you must use as_ptr method and return a raw pointer.
//! You can create them using PyString::from trait, from both &str and String.
//!
//! # Safety
//! When passed from Python you can convert from PyString to an owned string
//! (from\_ptr\_into\_string method) or to a &str slice (to\_str method), or
//! to a PyString reference (from\_ptr method). Those operations are unsafe
//! as they require dereferencing a raw pointer.
//!
//! # Examples
//!
//! ```
//! use rustypy::PyString;
//! let pystr = PyString::from("Hello world!");
//!
//! // prepare to return to Python:
//! let ptr = pystr.as_ptr();
//! // convert from raw pointer to an owned String
//! let rust_string = PyString::from_ptr_into_string(ptr);
//! ```
use std::ffi::CString;
use libc::c_char;

use std::convert::From;
use std::fmt;

/// An analog of a Python String.
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone)]
#[derive(Debug)]
pub struct PyString {
    _inner: CString,
}

impl PyString {
    /// Get a PyString from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyString) -> PyString {
        *(Box::from_raw(ptr))
    }
    /// Constructs an owned String from a PyString.
    pub fn to_string(&self) -> String {
        String::from(self._inner.to_str().unwrap())
    }
    /// Constructs an owned String from a raw pointer.
    pub unsafe fn from_ptr_to_string(ptr: *mut PyString) -> String {
        let pystr = *(Box::from_raw(ptr));
        String::from(pystr._inner.to_str().unwrap())
    }
    /// Returns PyString as a raw pointer. Use this whenever you want to return
    /// a PyString to Python.
    pub fn as_ptr(self) -> *mut PyString {
        Box::into_raw(Box::new(self))
    }
    /// Return a PyString from a raw char pointer.
    pub unsafe fn from_raw(ptr: *const c_char) -> PyString {
        PyString { _inner: CStr::from_ptr(ptr).to_owned() }
    }
}

impl fmt::Display for PyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<'a> From<&'a str> for PyString {
    /// Copies a string slice to a PyString.
    fn from(s: &'a str) -> PyString {
        PyString { _inner: CString::new(s).unwrap() }
    }
}

impl From<String> for PyString {
    /// Copies a String to a PyString.
    fn from(s: String) -> PyString {
        PyString { _inner: CString::new(s).unwrap() }
    }
}

/// Destructs the PyString, mostly to be used from Python.
#[no_mangle]
pub extern "C" fn pystring_free(ptr: *mut PyString) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

use std::ffi::CStr;
/// Creates a PyString wrapper from a raw c_char pointer
#[no_mangle]
pub extern "C" fn pystring_new(ptr: *const c_char) -> *mut PyString {
    let pystr = PyString { _inner: unsafe { CStr::from_ptr(ptr).to_owned() } };
    pystr.as_ptr()
}

/// Consumes the wrapper and returns a raw c_char pointer. Afterwards is not necessary
/// to destruct it as it has already been consumed.
#[no_mangle]
pub extern "C" fn pystring_get_str(ptr: *mut PyString) -> *const c_char {
    let pystr: PyString = unsafe { PyString::from_ptr(ptr) };
    pystr._inner.into_raw()
}
