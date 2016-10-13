#[macro_use]
extern crate rustypy;

use rustypy::{PyArg, PyList, PyString, PyBool};

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
    use rustypy::pytypes::pytuple;
    let e1 = unsafe { PyBool::from_ptr(pytuple::pytuple_extract_pybool(ptr, 0usize)) };
    assert_eq!(e1.to_bool(), false);
    let e2 = unsafe { PyString::from_ptr(pytuple::pytuple_extract_pystring(ptr, 1usize)) };
    assert_eq!(&(e2.to_string()), "test");
    let e3 = unsafe { pytuple::pytuple_extract_pyint(ptr, 2usize) };
    assert_eq!(e3, 55);

    let pytuple = unsafe { rustypy::PyTuple::from_ptr(ptr) };
    let unpacked = unpack_pytuple!(pytuple; (PyBool, PyString, I64,));
    assert_eq!((false, String::from("test"), 55i64), unpacked);
}
