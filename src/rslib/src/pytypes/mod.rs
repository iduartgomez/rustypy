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
}

mod tuple_macros {
/// This macro allows the construction of
/// [PyTuple](../rustypy/pytypes/pytuple/struct.PyTuple.html) types.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use rustypy::PyArg;
/// pytuple!(PyArg::I64(10), PyArg::F32(10.5));
/// # }
/// ```
///
#[macro_export]
macro_rules! pytuple {
    ( $( $elem:ident ),+ ) => {{
        use rustypy::PyTuple;
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
        tuple.pop().unwrap()
    }};
    ( $( $elem:expr ),+ ) => {{
        use rustypy::PyTuple;
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
        if cnt == 0 {}; // stub to remove warning...
        let t_len = tuple.len() - 1;
        for i in 1..(t_len + 1) {
            let idx = t_len - i;
            let last = tuple.pop().unwrap();
            let prev = tuple.get_mut(idx).unwrap();
            prev.next = Some(Box::new(last));
        }
        tuple.pop().unwrap()
    }};
}
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
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
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
