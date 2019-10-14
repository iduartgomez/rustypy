use rustypy::{PyArg, PyBool, PyList, PyTuple};
use std::iter::FromIterator;

#[no_mangle]
pub unsafe extern "C" fn python_bind_list2(list: *mut PyList) -> *mut PyList {
    let converted = unpack_pylist!(list; PyList{PyTuple{(I64, (F32, I64,),)}});
    assert_eq!(
        vec![(50i64, (1.0f32, 30i64)), (25i64, (0.5f32, 40i64))],
        converted
    );

    let v: Vec<PyTuple> = vec![
        pytuple!(PyArg::F64(0.5f64), PyArg::PyBool(PyBool::from(true))),
        pytuple!(PyArg::F64(-0.5f64), PyArg::PyBool(PyBool::from(false))),
    ];
    PyList::from_iter(v).into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn python_bind_nested1_t_n_ls(list: *mut PyList) -> *mut PyList {
    let converted = unpack_pylist!(list; PyList{PyList{PyTuple{(I64, (F32, I64,),)}}});
    assert_eq!(
        vec![
            vec![(50i64, (1.0f32, 30i64))],
            vec![(25i64, (0.5f32, 40i64))],
        ],
        converted
    );
    let mut v0 = Vec::new();
    for x in converted {
        let mut v1 = Vec::new();
        for (f1, (f2, f3)) in x {
            let t_e = pytuple!(
                PyArg::I64(f1),
                PyArg::PyTuple(pytuple!(PyArg::F32(f2), PyArg::I64(f3)).into_raw())
            );
            v1.push(t_e);
        }
        v0.push(v1);
    }
    PyList::from_iter(v0).into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn python_bind_nested2_t_n_ls(list: *mut PyList) -> *mut PyList {
    let mut unpacked = unpack_pylist!(list; PyList{PyTuple{({PyList{I64 => i64}}, F32,)}});
    assert_eq!(vec![(vec![1, 2, 3], 0.1), (vec![3, 2, 1], 0.2)], unpacked);
    unpacked.swap(0, 1);
    let mut v0 = Vec::new();
    for (f1, f2) in unpacked {
        let e = pytuple!(
            PyArg::PyList(PyList::from_iter(f1).into_raw()),
            PyArg::F32(f2)
        );
        v0.push(e);
    }
    PyList::from_iter(v0).into_raw()
}
