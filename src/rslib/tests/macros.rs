#[macro_use]
extern crate rustypy;

use rustypy::{PyArg, PyList, PyTuple, PyString, PyBool};

#[test]
fn test_unpack_pylist_macro() {
    use std::iter::FromIterator;
    let nested = Box::new(PyList::from_iter(vec![
        pytuple!(PyArg::PyList(Box::new(PyList::from_iter(vec![1i32, 2, 3]))),
            PyArg::F32(0.1)),
        pytuple!(PyArg::PyList(Box::new(PyList::from_iter(vec![3i32, 2, 1]))),
            PyArg::F32(0.2))
    ]));
    let unpacked = unpack_pylist!(nested; PyList{PyTuple{({PyList{I32 => i32}}, F32,)}});
    assert_eq!(vec![(vec![1, 2, 3], 0.1), (vec![3, 2, 1], 0.2)], unpacked);
}

#[test]
fn test_pytuple_macros() {
    let ptr = pytuple!(PyArg::PyBool(PyBool::from(false)),
                       PyArg::PyString(PyString::from("test")),
                       PyArg::I64(55i64))
            .as_ptr();

    let mut pytuple = unsafe { PyTuple::from_ptr(ptr) };
    let unpacked = unpack_pytuple!(pytuple; (PyBool, PyString, I64,));
    assert_eq!((false, String::from("test"), 55i64), unpacked);
}
