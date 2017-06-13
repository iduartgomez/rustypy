//! Types for interfacing with Python.

use libc::{size_t, c_char};

#[doc(hidden)]
#[macro_export]
macro_rules! _rustypy_abort_xtract_fail {
    ( $msg:expr ) => {{
        use std::process;
        use std::io::{Write, stderr, stdout};

        fn write<T: Write>(mut handle: T) {
            write!(&mut handle, "\nrustypy: failed abrupty!").unwrap();
            write!(&mut handle,
                "rustypy: aborted process, tried to extract one type, but found an other instead:\n \
                {}\n", $msg).unwrap();
            handle.flush().unwrap();
        }

        let err = stderr();
        write(err);
        let out = stdout();
        write(out);
        process::exit(1);
    }}
}

pub mod pystring;
pub mod pybool;
pub mod pytuple;
pub mod pylist;
pub mod pydict;

use self::pybool::PyBool;
use self::pystring::PyString;
use self::pylist::PyList;
use self::pytuple::PyTuple;

/// Enum type used to construct PyTuple and PyList types. All the kinds supported in Python
/// are included here.
///
/// In Python, conversion of floats default to double precision unless explicitly stated
/// adding the Float custom rustypy type to the return type signature.
///
/// ```Python
///     from rustypy.rswrapper import Double, Float
///     bindings.my_binded_func.restype = Float
///     bindings.my_binded_func.restype = Double
/// ```
///
/// Likewise, all 'int' types are converted to signed 64-bit integers by default.
#[derive(Clone, Debug, PartialEq)]
pub enum PyArg {
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    F32(f32),
    F64(f64),
    PyBool(PyBool),
    PyString(PyString),
    PyTuple(Box<PyTuple>),
    PyList(Box<PyList>),
    PyDict(*mut size_t),
    None,
}

impl PyArg {
    pub fn as_ptr(self) -> *mut PyArg {
        Box::into_raw(Box::new(self))
    }
}

// Conversions from <T> to PyArg
impl<'a> From<&'a str> for PyArg {
    fn from(a: &str) -> PyArg {
        PyArg::PyString(PyString::from(a))
    }
}

impl From<String> for PyArg {
    fn from(a: String) -> PyArg {
        PyArg::PyString(PyString::from(a))
    }
}

impl From<PyString> for PyArg {
    fn from(a: PyString) -> PyArg {
        PyArg::PyString(a)
    }
}

impl From<bool> for PyArg {
    fn from(a: bool) -> PyArg {
        PyArg::PyBool(PyBool::from(a))
    }
}

impl<'a> From<&'a bool> for PyArg {
    fn from(a: &'a bool) -> PyArg {
        PyArg::PyBool(PyBool::from(a))
    }
}

impl From<PyBool> for PyArg {
    fn from(a: PyBool) -> PyArg {
        PyArg::PyBool(a)
    }
}

impl From<i8> for PyArg {
    fn from(a: i8) -> PyArg {
        PyArg::I8(a)
    }
}

impl From<u8> for PyArg {
    fn from(a: u8) -> PyArg {
        PyArg::U8(a)
    }
}

impl From<i16> for PyArg {
    fn from(a: i16) -> PyArg {
        PyArg::I16(a)
    }
}

impl From<u16> for PyArg {
    fn from(a: u16) -> PyArg {
        PyArg::U16(a)
    }
}

impl From<i32> for PyArg {
    fn from(a: i32) -> PyArg {
        PyArg::I32(a)
    }
}

impl From<u32> for PyArg {
    fn from(a: u32) -> PyArg {
        PyArg::U32(a)
    }
}

impl From<i64> for PyArg {
    fn from(a: i64) -> PyArg {
        PyArg::I64(a)
    }
}

impl From<u64> for PyArg {
    fn from(a: u64) -> PyArg {
        PyArg::U64(a)
    }
}

impl From<f32> for PyArg {
    fn from(a: f32) -> PyArg {
        PyArg::F32(a)
    }
}

impl From<f64> for PyArg {
    fn from(a: f64) -> PyArg {
        PyArg::F64(a)
    }
}

impl From<PyList> for PyArg {
    fn from(a: PyList) -> PyArg {
        PyArg::PyList(Box::new(a))
    }
}

impl From<PyTuple> for PyArg {
    fn from(a: PyTuple) -> PyArg {
        PyArg::PyTuple(Box::new(a))
    }
}

// From<PyDict<K, PyArg>> is implemented in mod pydict due to private K bound

impl<T> From<Vec<T>> for PyArg
    where PyArg: From<T>
{
    fn from(a: Vec<T>) -> PyArg {
        PyArg::PyList(Box::new(PyList::from(a)))
    }
}

// Conversions from PyArg to <T>
impl From<PyArg> for u8 {
    fn from(a: PyArg) -> u8 {
        match a {
            PyArg::U8(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a u8 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for i8 {
    fn from(a: PyArg) -> i8 {
        match a {
            PyArg::I8(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a i8 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for u16 {
    fn from(a: PyArg) -> u16 {
        match a {
            PyArg::U16(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a u16 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for i16 {
    fn from(a: PyArg) -> i16 {
        match a {
            PyArg::I16(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a i16 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for u32 {
    fn from(a: PyArg) -> u32 {
        match a {
            PyArg::U32(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a u32 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for i32 {
    fn from(a: PyArg) -> i32 {
        match a {
            PyArg::I32(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a i32 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for u64 {
    fn from(a: PyArg) -> u64 {
        match a {
            PyArg::U64(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a u64 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for i64 {
    fn from(a: PyArg) -> i64 {
        match a {
            PyArg::I64(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a i64 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for f32 {
    fn from(a: PyArg) -> f32 {
        match a {
            PyArg::F32(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a f32 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for f64 {
    fn from(a: PyArg) -> f64 {
        match a {
            PyArg::F64(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a f64 while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for PyString {
    fn from(a: PyArg) -> PyString {
        match a {
            PyArg::PyString(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a PyString while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for String {
    fn from(a: PyArg) -> String {
        match a {
            PyArg::PyString(v) => v.to_string(),
            _ => _rustypy_abort_xtract_fail!("expected a PyString while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for PyBool {
    fn from(a: PyArg) -> PyBool {
        match a {
            PyArg::PyBool(v) => v,
            _ => _rustypy_abort_xtract_fail!("expected a PyBool while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for PyTuple {
    fn from(a: PyArg) -> PyTuple {
        match a {
            PyArg::PyTuple(v) => *v,
            _ => _rustypy_abort_xtract_fail!("expected a PyTuple while destructuring PyArg enum")
        }
    }
}

impl From<PyArg> for PyList {
    fn from(a: PyArg) -> PyList {
        match a {
            PyArg::PyList(v) => *v,
            _ => _rustypy_abort_xtract_fail!("expected a PyList while destructuring PyArg enum")
        }
    }
}


// From types:

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_int(e: i64) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::I64(e)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_ulonglong(e: u64) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::U64(e)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_float(e: f32) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::F32(e)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_double(e: f64) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::F64(e)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_bool(e: i8) -> *mut PyArg {
    let e = PyBool::from(e);
    Box::into_raw(Box::new(PyArg::PyBool(e)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_str(e: *const c_char) -> *mut PyArg {
    let e = unsafe { PyString::from_raw(e) };
    Box::into_raw(Box::new(PyArg::PyString(e)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_pytuple(e: *mut PyTuple) -> *mut PyArg {
    let e = unsafe { PyTuple::from_ptr(e) };
    Box::into_raw(Box::new(PyArg::PyTuple(Box::new(e))))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_pylist(e: *mut PyList) -> *mut PyArg {
    let e = unsafe { PyList::from_ptr(e) };
    Box::into_raw(Box::new(PyArg::PyList(Box::new(e))))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_from_pydict(e: *mut size_t) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::PyDict(e)))
}

// Extract owned args, no copies:
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_int(e: *mut PyArg) -> i64 {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::I64(val) => val,
        PyArg::I32(val) => val as i64,
        PyArg::I16(val) => val as i64,
        PyArg::I8(val) => val as i64,
        PyArg::U32(val) => val as i64,
        PyArg::U16(val) => val as i64,
        PyArg::U8(val) => val as i64,
        _ => {
            _rustypy_abort_xtract_fail!("failed while trying to extract an integer type of i64 or \
                                        less")
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_ulonglong(e: *mut PyArg) -> u64 {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::U64(val) => val,
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract an u64"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_float(e: *mut PyArg) -> f32 {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::F32(val) => val,
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract an f32"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_double(e: *mut PyArg) -> f64 {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::F64(val) => val,
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract an f64"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_bool(e: *mut PyArg) -> *mut PyBool {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyBool(val) => val.as_ptr(),
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract a PyBool"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_str(e: *mut PyArg) -> *mut PyString {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyString(val) => val.as_ptr(),
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract a PyString"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_tuple(e: *mut PyArg) -> *mut PyTuple {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyTuple(val) => (*val).as_ptr(),
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract a PyTuple"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_list(e: *mut PyArg) -> *mut PyList {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyList(val) => (*val).as_ptr(),
        _ => _rustypy_abort_xtract_fail!("failed while trying to extract a PyList"),
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pyarg_extract_owned_dict(e: *mut PyArg) -> *mut size_t {
    unsafe {
        let e = *(Box::from_raw(e));
        match e {
            PyArg::PyDict(val) => val,
            _ => _rustypy_abort_xtract_fail!("failed while trying to extract a PyDict"),
        }
    }
}
