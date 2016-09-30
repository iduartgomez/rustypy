//! An analog of a Python tuple, will accept an undefined number of other supported types.
//!
//! You can construct it using the [pytuple!](../../macro.pytuple!.html) macro, ie:
//!
//! ```
//! # #[macro_use] extern crate rustypy;
//! # fn main(){
//! pytuple!(PyArg::I64(10), PyArg::F32(10.5))
//! # }
//! ```
//!
//! You must pass the variety of the argument using the PyArg enum.
//!
//! When extracting from Python with the FFI elements are copied, not moved, and when free'd
//! all the original elements are dropped
use libc;
use pytypes::{PyArg, PyBool, PyString};

/// An analog of a Python tuple, will accept an undefined number of other supported types.
///
/// Read the [module docs](index.html) for more information.
#[derive(Debug)]
pub struct PyTuple {
    pub elem: PyArg,
    pub idx: usize,
    pub next: Option<Box<PyTuple>>,
}

extern "C" fn _abort_msg() {
    use std::io::{self, Write};
    let mut output = io::stdout();
    output.write(b"rustypy failed abrupty!").unwrap();
    output.flush().unwrap();
}

macro_rules! abort_on_extraction_fail {
    ($t:ident) => {{
        use std::io::{self, Write};
        let msg = format!(
            "rustypy: aborted process, tried to extract one type, but found {:?} instead", $t);
        //let msg = CString::new(msg.as_str()).unwrap().as_ptr();
        let mut output = io::stdout();
        output.write(msg.as_bytes()).unwrap();
        libc::atexit(_abort_msg);
        libc::exit(1)
    }};
}

impl<'a> PyTuple {
    fn get_element(&self, idx: usize) -> Result<&PyTuple, &str> {
        if idx == self.idx {
            Ok(self)
        } else {
            match self.next {
                Some(ref e) => e.get_element(idx),
                None => Err("PyTuple index out of range."),
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

/// This macro allows the construction of [PyTuple](../rustypy/pytypes/struct.PyTuple.html) types.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// pytuple!(PyArg::I64(10), PyArg::F32(10.5))
/// # }
/// ```
///
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
        PyArg::I32(val) => val as i64,
        PyArg::I16(val) => val as i64,
        PyArg::I8(val) => val as i64,
        PyArg::U32(val) => val as i64,
        PyArg::U16(val) => val as i64,
        PyArg::U8(val) => val as i64,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyBool(ptr: *mut PyTuple, index: usize) -> *mut PyBool {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::PyBool(ref val) => val.clone().as_ptr(),
        _ => abort_on_extraction_fail!(elem),
    }
}


#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyFloat(ptr: *mut PyTuple, index: usize) -> f32 {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::F32(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyDouble(ptr: *mut PyTuple, index: usize) -> f64 {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::F64(val) => val,
        _ => abort_on_extraction_fail!(elem),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyTuple_extractPyString(ptr: *mut PyTuple, index: usize) -> *mut PyString {
    let tuple = &*ptr;
    let elem = PyTuple::get_element(tuple, index).unwrap();
    match elem.elem {
        PyArg::PyString(ref val) => val.clone().as_ptr(),
        _ => abort_on_extraction_fail!(elem),
    }
}
