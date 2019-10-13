//! Analog to a Python boolean type.
//!
//! It supports & and | operators, and comparison to Rust bool types.
//! To return to Python use the ```into_raw``` method and return a raw pointer.
//!
//! # Safety
//! You can convert a raw pointer to a bool type with ```from_ptr_into_bool``` method,
//! or to a ```&PyBool``` with ```from_ptr``` method. Those operations are unsafe as they require
//! dereferencing a raw pointer.
//!
//! # Examples
//!
//! ```
//! use rustypy::PyBool;
//! let pybool = PyBool::from(true);
//! assert_eq!(pybool, true);
//!
//! // prepare to return to Python:
//! let ptr = pybool.into_raw();
//! // convert from raw pointer to a bool
//! let rust_bool = unsafe { PyBool::from_ptr_into_bool(ptr) };
//! ```
use libc::c_char;

use std::convert::From;
use std::ops::{BitAnd, BitOr, Not};

/// Analog to a Python boolean type.
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct PyBool {
    val: i8,
}

impl PyBool {
    /// Get a PyBool from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyBool) -> PyBool {
        *(Box::from_raw(ptr))
    }

    /// Creates a bool from a raw pointer to a PyBool.
    pub unsafe fn from_ptr_into_bool(ptr: *mut PyBool) -> bool {
        let ptr: &PyBool = &*ptr;
        match ptr.val {
            0 => false,
            _ => true,
        }
    }

    /// Conversion from PyBool to bool.
    pub fn to_bool(self) -> bool {
        match self.val {
            0 => false,
            _ => true,
        }
    }

    /// Returns PyBool as a raw pointer. Use this whenever you want to return
    /// a PyBool to Python.
    pub fn into_raw(self) -> *mut PyBool {
        Box::into_raw(Box::new(self))
    }

    /// Sets value of the underlying bool
    pub fn load(&mut self, v: bool) {
        if v {
            self.val = 1
        } else {
            self.val = 0
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pybool_free(ptr: *mut PyBool) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pybool_new(val: c_char) -> *mut PyBool {
    let val = match val {
        0 => 0,
        _ => 1,
    };
    let pystr = PyBool { val };
    pystr.into_raw()
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pybool_get_val(ptr: *mut PyBool) -> i8 {
    let pybool = &*ptr;
    pybool.val
}

impl From<PyBool> for bool {
    fn from(b: PyBool) -> bool {
        b.to_bool()
    }
}

impl From<bool> for PyBool {
    fn from(b: bool) -> PyBool {
        let val = if b { 1 } else { 0 };
        PyBool { val }
    }
}

impl<'a> From<&'a bool> for PyBool {
    fn from(b: &'a bool) -> PyBool {
        let val = if *b { 1 } else { 0 };
        PyBool { val }
    }
}

impl From<i8> for PyBool {
    fn from(b: i8) -> PyBool {
        let val = match b {
            0 => 0,
            _ => 1,
        };
        PyBool { val }
    }
}

impl PartialEq<bool> for PyBool {
    fn eq(&self, other: &bool) -> bool {
        (self.val == 0 && !(*other)) || (self.val == 1 && *other)
    }
}

impl<'a> PartialEq<bool> for &'a PyBool {
    fn eq(&self, other: &bool) -> bool {
        (self.val == 0 && !(*other)) || (self.val == 1 && *other)
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
