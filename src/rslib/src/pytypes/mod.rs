//! Types for interfacing with Python.

use libc::{size_t, c_char};

use std::hash::Hash;
use std::collections::HashMap;
use std::convert::AsRef;

#[doc(hidden)]
#[macro_export]
macro_rules! _rustypy_abort_xtract_fail {
    ( $msg:expr ) => {{
        let msg = $msg;
        _rustypy_abort_xtract_fail!(var msg);
    }};
    ( var $msg:ident ) => {{
        use std::process;
        use std::io::{Write, stderr, stdout};

        fn write<T: Write, M: ::std::fmt::Display>(mut handle: T, msg: &M) {
            write!(&mut handle, "\nrustypy: failed abrupty!").unwrap();
            write!(&mut handle,
                "rustypy: aborted process, tried to extract one type, but found an other \
                 instead:\n {}\n", msg).unwrap();
            handle.flush().unwrap();
        }

        let err = stderr();
        write(err, &$msg);
        let out = stdout();
        write(out, &$msg);
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
use self::pytuple::PyTuple;
use self::pylist::PyList;
use self::pydict::{PyDict, PyDictKey};

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

macro_rules! pyarg_conversions {
    ($type:ty; $variant:path; $repr:expr) => {
        impl AsRef<$type> for PyArg {
            fn as_ref(&self) -> &$type {
                match *self {
                    $variant(ref v) => v,
                    _ => {
                        let msg = format!("expected a {} while destructuring PyArg enum", $repr);
                        _rustypy_abort_xtract_fail!(var msg);
                    }
                }
            }
        }

        impl From<$type> for PyArg {
            fn from(a: $type) -> PyArg {
                $variant(a)
            }
        }

        impl From<PyArg> for $type {
            fn from(a: PyArg) -> $type {
                match a {
                    $variant(v) => v,
                    _ => {
                        let msg = format!("expected a {} while destructuring PyArg enum", $repr);
                        _rustypy_abort_xtract_fail!(var msg);
                    }
                }
            }
        }
    };
    (BOXED $type:ty; $variant:path; $repr:expr) => {
        impl AsRef<$type> for PyArg {
            fn as_ref(&self) -> &$type {
                match *self {
                    $variant(ref v) => &**v,
                    _ => {
                        let msg = format!("expected a {} while destructuring PyArg enum", $repr);
                        _rustypy_abort_xtract_fail!(var msg);
                    }
                }
            }
        }

        impl From<$type> for PyArg {
            fn from(a: $type) -> PyArg {
                $variant(Box::new(a))
            }
        }

        impl From<PyArg> for $type {
            fn from(a: PyArg) -> $type {
                match a {
                    $variant(v) => *v,
                    _ => {
                        let msg = format!("expected a {} while destructuring PyArg enum", $repr);
                        _rustypy_abort_xtract_fail!(var msg);
                    }
                }
            }
        }
    }
}

pyarg_conversions!(i8; PyArg::I8; "i8");
pyarg_conversions!(i16; PyArg::I16; "i16");
pyarg_conversions!(i32; PyArg::I32; "i32");
pyarg_conversions!(i64; PyArg::I64; "i64");
pyarg_conversions!(u8; PyArg::U8; "u8");
pyarg_conversions!(u16; PyArg::U16; "u16");
pyarg_conversions!(u32; PyArg::U32; "u32");
pyarg_conversions!(u64; PyArg::U64; "u64");
pyarg_conversions!(f32; PyArg::F32; "f32");
pyarg_conversions!(f64; PyArg::F64; "f64");
pyarg_conversions!(PyBool; PyArg::PyBool; "PyBool");
pyarg_conversions!(PyString; PyArg::PyString; "PyString");
pyarg_conversions!(BOXED PyTuple; PyArg::PyTuple; "PyTuple");
pyarg_conversions!(BOXED PyList; PyArg::PyList; "PyList");

impl<K> AsRef<PyDict<K>> for PyArg
    where K: Eq + Hash + PyDictKey
{
    fn as_ref(&self) -> &PyDict<K> {
        match *self {
            PyArg::PyDict(dict) => unsafe { &*(dict as *mut PyDict<K>) as &PyDict<K> },
            _ => _rustypy_abort_xtract_fail!("expected a PyDict while destructuring PyArg enum"),
        }
    }
}

// Conversions: PyArg from <T>

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

impl<T> From<Vec<T>> for PyArg
    where PyArg: From<T>
{
    fn from(a: Vec<T>) -> PyArg {
        PyArg::PyList(Box::new(PyList::from(a)))
    }
}

impl<K> From<PyDict<K>> for PyArg
    where K: Eq + Hash + PyDictKey
{
    fn from(a: PyDict<K>) -> PyArg {
        PyArg::PyDict(a.as_ptr())
    }
}

impl<K, V> From<HashMap<K, V>> for PyArg
    where PyArg: From<V>,
          K: Eq + Hash + PyDictKey
{
    fn from(a: HashMap<K, V>) -> PyArg {
        let dict = PyDict::from(a);
        PyArg::PyDict(dict.as_ptr())
    }
}

// Conversions from PyArg to <T>

impl From<PyArg> for String {
    fn from(a: PyArg) -> String {
        match a {
            PyArg::PyString(v) => v.to_string(),
            _ => _rustypy_abort_xtract_fail!("expected a PyString while destructuring PyArg enum"),
        }
    }
}

impl<K> From<PyArg> for PyDict<K>
    where K: Eq + Hash + PyDictKey
{
    fn from(a: PyArg) -> PyDict<K> {
        match a {
            PyArg::PyDict(v) => unsafe { *(Box::from_raw(v as *mut PyDict<K>)) },
            _ => _rustypy_abort_xtract_fail!("expected a PyDict while destructuring PyArg enum"),
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
