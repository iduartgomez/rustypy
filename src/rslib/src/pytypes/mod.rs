//! Types for interfacing with Python.

// private macros
#[doc(hidden)]
macro_rules! abort_on_extraction_fail {
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
        //let msg = CString::new(msg.as_str()).unwrap().as_ptr();
        let mut output = io::stdout();
        output.write(msg.as_bytes()).unwrap();
        libc::atexit(_abort_msg);
        libc::exit(1)
    }};
}

pub mod pystring;
pub mod pybool;
pub mod pytuple;
pub mod pylist;

pub use self::pybool::PyBool;
pub use self::pystring::PyString;
pub use self::pytuple::PyTuple;
pub use self::pylist::PyList;

/// Enum type used to construct PyTuple types. All the kinds supported in Python
/// are included here.
///
/// In Python, conversion of floats default to double precision unless explicitly stated
/// Adding the Float custom rustypy type to the return type signature.
///
/// ```Python
///     from rustypy.rswrapper import Double, Float
///     bindings.my_binded_func.restype = Float
///     bindings.my_binded_func.restype = Double
/// ```
///
#[derive(Debug)]
#[derive(Clone)]
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
}

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

use libc::c_char;

#[no_mangle]
pub extern "C" fn pyarg_from_str(e: *const c_char) -> *mut PyArg {
    let e = unsafe { PyString::from_raw(e) };
    Box::into_raw(Box::new(PyArg::PyString(e)))
}

#[no_mangle]
pub extern "C" fn pyarg_from_bool(e: i8) -> *mut PyArg {
    let e = PyBool::from(e);
    Box::into_raw(Box::new(PyArg::PyBool(e)))
}
