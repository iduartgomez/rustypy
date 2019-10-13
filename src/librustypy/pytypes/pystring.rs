//! An analog of a Python String.
//!
//! To return to Python you must use ```into_raw``` method and return a raw pointer.
//! You can create them using the ```from``` trait method, from both ```&str``` and ```String```.
//!
//! # Safety
//! When passed from Python you can convert from PyString to an owned string
//! (```from_ptr_into_string``` method) or to a ```&str``` slice (to_str method), or
//! to a PyString reference (```from_ptr``` method). Those operations are unsafe
//! as they require dereferencing a raw pointer.
//!
//! # Examples
//!
//! ```
//! use rustypy::PyString;
//! let pystr = PyString::from("Hello world!");
//!
//! // prepare to return to Python:
//! let ptr = pystr.into_raw();
//! // convert from raw pointer to an owned String
//! let rust_string = unsafe { PyString::from_ptr_to_string(ptr) };
//! ```
use libc::c_char;
use std::ffi::CString;

use std::convert::From;
use std::fmt;

/// An analog of a Python string.
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PyString {
    _inner: CString,
}

impl PyString {
    /// Get a PyString from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyString) -> PyString {
        *Box::from_raw(ptr)
    }
    /// Constructs an owned String from a raw pointer.
    pub unsafe fn from_ptr_to_string(ptr: *mut PyString) -> String {
        let pystr = *(Box::from_raw(ptr));
        String::from(pystr._inner.to_str().unwrap())
    }
    /// Returns PyString as a raw pointer. Use this whenever you want to return
    /// a PyString to Python.
    pub fn into_raw(self) -> *mut PyString {
        Box::into_raw(Box::new(self))
    }
    /// Return a PyString from a raw char pointer.
    pub unsafe fn from_raw(ptr: *const c_char) -> PyString {
        PyString {
            _inner: CStr::from_ptr(ptr).to_owned(),
        }
    }
}

impl fmt::Display for PyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from(self._inner.to_str().unwrap()))
    }
}

impl<'a> From<&'a str> for PyString {
    /// Copies a string slice to a PyString.
    fn from(s: &'a str) -> PyString {
        PyString {
            _inner: CString::new(s).unwrap(),
        }
    }
}

impl From<String> for PyString {
    /// Copies a String to a PyString.
    fn from(s: String) -> PyString {
        PyString {
            _inner: CString::new(s).unwrap(),
        }
    }
}

impl From<PyString> for String {
    fn from(s: PyString) -> String {
        s.to_string()
    }
}

/// Destructs the PyString, mostly to be used from Python.
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pystring_free(ptr: *mut PyString) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

use std::ffi::CStr;
/// Creates a PyString wrapper from a raw c_char pointer
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pystring_new(ptr: *const c_char) -> *mut PyString {
    let pystr = PyString {
        _inner: CStr::from_ptr(ptr).to_owned(),
    };
    pystr.into_raw()
}

/// Consumes the wrapper and returns a raw c_char pointer. Afterwards is not necessary
/// to destruct it as it has already been consumed.
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pystring_get_str(ptr: *mut PyString) -> *const c_char {
    let pystr: PyString = PyString::from_ptr(ptr);
    pystr._inner.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pystring_operations() {
        let source = "test string";
        let owned_pystr = PyString::from(source).into_raw();
        let back_from_py = unsafe { PyString::from_ptr_to_string(owned_pystr) };
        assert_eq!(back_from_py, "test string");
        {
            String::from(source);
        }
    }
}
