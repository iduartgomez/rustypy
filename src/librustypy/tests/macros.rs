extern crate rustypy;
use rustypy::*;

#[test]
fn unpack_pylist_macro() {
    use std::iter::FromIterator;
    let nested = PyList::from_iter(vec![
        pytuple!(
            PyArg::PyList(PyList::from_iter(vec![1i32, 2, 3]).into_raw()),
            PyArg::F32(0.1)
        ),
        pytuple!(
            PyArg::PyList(PyList::from_iter(vec![3i32, 2, 1]).into_raw()),
            PyArg::F32(0.2)
        ),
    ])
    .into_raw();
    let unpacked = unpack_pytype!(nested; PyList{PyTuple{({PyList{I32 => i32}}, F32,)}});
    assert_eq!(vec![(vec![1, 2, 3], 0.1), (vec![3, 2, 1], 0.2)], unpacked);
}

#[test]
fn pytuple_macros() {
    let pytuple = pytuple!(
        PyArg::PyBool(PyBool::from(false)),
        PyArg::PyString(PyString::from("test")),
        PyArg::I64(55i64)
    )
    .into_raw();
    let unpacked = unpack_pytype!(pytuple; (PyBool, PyString, I64,));
    assert_eq!((false, String::from("test"), 55i64), unpacked);
}
