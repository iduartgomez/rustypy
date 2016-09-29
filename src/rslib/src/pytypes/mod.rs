//! Types for interfacing with Python.
pub mod pystring;
pub mod pybool;
pub mod pytuple;

pub use self::pybool::PyBool;
pub use self::pystring::PyString;
pub use self::pytuple::PyTuple;

#[derive(Debug)]
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
    PyTuple(*mut PyTuple),
}
