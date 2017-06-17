extern crate libc;
extern crate cpython;

extern crate rustypy;

use std::collections::HashMap;

use libc::c_long;
use cpython::{Python, ToPyObject, PythonObject, PyObject, PyLong, PyString};

mod test_package;

#[test]
fn basics_nested_types() {
    use test_package::basics::rustypy_pybind::PyModules;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let module: PyModules = PyModules::new(&py);
    let basics = module.nested_types;

    // Vec<(c_double, bool)> -> Vec<String>
    let arg = vec![(0.2, true), (0.3, true), (0.5, true)];
    let answ = basics.list1(arg);
    assert_eq!(answ, vec![String::from("passed")]);

    // HashMap<String, c_long> -> HashMap<String, c_long>
    let mut dict = HashMap::new();
    dict.insert(String::from("first_key"), 1);
    let answ = basics.dict1(dict);
    let mut cmp = HashMap::new();
    cmp.insert(String::from("first_key"), 2);
    assert_eq!(answ, cmp);

    // HashMap<String, (String, bool)> -> HashMap<String, (String, bool)>
    let mut dict = HashMap::new();
    dict.insert(String::from("first_key"), (String::from("first_key"), true));
    let answ = basics.dict2(dict);
    let mut cmp = HashMap::new();
    cmp.insert(String::from("first_key"), (String::from("first_key"), true));
    assert_eq!(answ, cmp);

    // (c_long, (c_double, bool)) -> (c_long, (String, bool), c_double)
    let arg = (1, (0.5, true));
    let answ = basics.cmpd_tuple(arg);
    assert_eq!((1, (String::from("passed"), true), 0.0), answ);

    // Vec<((String, bool), PyObject)> -> Vec<(c_long, bool)>
    let a1: PyLong = 0.to_py_object(py);
    let a2: PyString = "str".to_py_object(py);
    let arg: Vec<((String, bool), PyObject)> =
        vec![((String::from("first"), true), a1.into_object()),
             ((String::from("second"), true), a2.into_object())];
    let answ = basics.cmpd_list_and_tuple(arg);
    assert_eq!(answ, vec![(0, false), (1, true)]);

    // Vec<(c_long, bool)>, Vec<c_long> -> Vec<(Vec<c_long>, c_double)>
    let arg1 = vec![(1, true), (0, false)];
    let arg2 = vec![0];
    let answ = basics.cmpd_list(arg1, arg2);
    let cmp = vec![(vec![1], 1.0)];
    assert_eq!(cmp, answ);

    // -> HashMap<String, HashMap<c_long, (String, bool)>>
    let answ = basics.cmpd_dict();
    let mut d1 = HashMap::new();
    let mut d2 = HashMap::new();
    d2.insert(0, (String::from("passed"), true));
    d1.insert(String::from("passed"), d2);
    assert_eq!(answ, d1);

    // -> Vec<HashMap<c_long, (String, bool)>>
    let answ = basics.cmpd_list_and_dict();
    let mut d1 = HashMap::new();
    d1.insert(0, (String::from("passed"), true));
    let l1 = vec![d1];
    assert_eq!(answ, l1);

    // -> HashMap<c_long, Vec<c_double>>
    let answ = basics.cmpd_dict_and_ls();
    let mut d1 = HashMap::new();
    d1.insert(0, vec![0.0, 1.0, 2.0, 3.0]);
    assert_eq!(answ, d1);

    // Generics:
    let arg: PyLong = 0.to_py_object(py);
    let answ = basics.generic1(arg.into_object());
    assert_eq!(0, answ.extract::<c_long>(py).unwrap());

    /*
    let a: PyLong = 0.to_py_object(py);
    let b: PyString = "second".to_py_object(py);
    let arg: Vec<PyObject> = vec![a.into_object(), b.into_object()];
    let answ = basics.generic2(arg);
    assert_eq!(answ.len(), 1);
    assert_eq!(&answ.get(0).unwrap().extract::<String>(py).unwrap(),
               "success");
    */
}
