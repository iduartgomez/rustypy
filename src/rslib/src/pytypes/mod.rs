//! Types for interfacing with Python.

#[doc(hidden)]
macro_rules! _abort_xtract_fail {
    ($t:ident) => {{
        extern "C" fn _abort_msg() {
            use std::io::{self, Write};
            let mut output = io::stdout();
            output.write(b"rustypy failed abrupty!").unwrap();
            output.flush().unwrap();
        }

        use std::io::{self, Write};
        let msg = format!(
            "rustypy: aborted process, tried to extract one type, \
            but found {:?} instead", $t);
        let mut output = io::stdout();
        output.write(msg.as_bytes()).unwrap();
        libc::atexit(_abort_msg);
        libc::exit(1)
    }};
    () => {{
        panic!("rustypy: panicked, tried to extract the wrong type")
    }}
}

pub mod pystring;
pub mod pybool;
pub mod pytuple;
pub mod pylist;
pub mod pydict;

pub use self::pystring::PyString;
pub use self::pybool::PyBool;
pub use self::pytuple::PyTuple;
pub use self::pylist::PyList;
pub use self::pydict::PyDict;

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
    U32(u32),
    U16(u16),
    U8(u8),
    F32(f32),
    F64(f64),
    PyBool(PyBool),
    PyString(PyString),
    PyTuple(Box<PyTuple>),
    PyList(Box<PyList>),
    None
}

// From types:

#[no_mangle]
pub extern "C" fn pyarg_from_int(e: i64) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::I64(e)))
}

#[no_mangle]
pub extern "C" fn pyarg_from_float(e: f32) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::F32(e)))
}

#[no_mangle]
pub extern "C" fn pyarg_from_double(e: f64) -> *mut PyArg {
    Box::into_raw(Box::new(PyArg::F64(e)))
}


#[no_mangle]
pub extern "C" fn pyarg_from_bool(e: i8) -> *mut PyArg {
    let e = PyBool::from(e);
    Box::into_raw(Box::new(PyArg::PyBool(e)))
}

use libc::c_char;

#[no_mangle]
pub extern "C" fn pyarg_from_str(e: *const c_char) -> *mut PyArg {
    let e = unsafe { PyString::from_raw(e) };
    Box::into_raw(Box::new(PyArg::PyString(e)))
}

#[no_mangle]
pub extern "C" fn pyarg_from_pytuple(e: *mut PyTuple) -> *mut PyArg {
    let e = unsafe { PyTuple::from_ptr(e) };
    Box::into_raw(Box::new(PyArg::PyTuple(Box::new(e))))
}

#[no_mangle]
pub extern "C" fn pyarg_from_pylist(e: *mut PyList) -> *mut PyArg {
    let e = unsafe { PyList::from_ptr(e) };
    Box::into_raw(Box::new(PyArg::PyList(Box::new(e))))
}

// Extract owned args, no copies:
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
        _ => _abort_xtract_fail!(),
    }
}

#[no_mangle]
pub extern "C" fn pyarg_extract_owned_float(e: *mut PyArg) -> f32 {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::F32(val) => val,
        _ => _abort_xtract_fail!(),
    }
}

#[no_mangle]
pub extern "C" fn pyarg_extract_owned_double(e: *mut PyArg) -> f64 {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::F64(val) => val,
        _ => _abort_xtract_fail!(),
    }
}

#[no_mangle]
pub extern "C" fn pyarg_extract_owned_bool(e: *mut PyArg) -> *mut PyBool {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyBool(val) => val.as_ptr(),
        _ => _abort_xtract_fail!(),
    }
}

#[no_mangle]
pub extern "C" fn pyarg_extract_owned_str(e: *mut PyArg) -> *mut PyString {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyString(val) => val.as_ptr(),
        _ => _abort_xtract_fail!(),
    }
}

#[no_mangle]
pub extern "C" fn pyarg_extract_owned_tuple(e: *mut PyArg) -> *mut PyTuple {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyTuple(val) => (*val).as_ptr() ,
        _ => _abort_xtract_fail!(),
    }
}

#[no_mangle]
pub extern "C" fn pyarg_extract_owned_list(e: *mut PyArg) -> *mut PyList {
    let e = unsafe { *(Box::from_raw(e)) };
    match e {
        PyArg::PyList(val) => (*val).as_ptr(),
        _ => _abort_xtract_fail!(),
    }
}
