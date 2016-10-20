use libc::size_t;
use pytypes::{PyArg, PyBool, PyString, PyTuple};

use std::collections::HashMap;
use std::collections::hash_map::Drain;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ptr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyDict<K, PyArg>
    where K: Eq + Hash
{
    table: HashMap<K, PyArg>,
}

use self::key_bound::PyDictKey;

impl<K> PyDict<K, PyArg>
    where K: Eq + Hash + PyDictKey
{
    pub fn new() -> PyDict<K, PyArg> {
        PyDict { table: HashMap::new() }
    }
    pub fn insert(&mut self, k: K, v: PyArg) -> Option<PyArg> {
        self.table.insert(k, v)
    }
    pub fn remove(&mut self, k: &K) -> Option<PyArg> {
        self.table.remove(k)
    }
    pub fn get(&mut self, k: &K) -> Option<&PyArg> {
        self.table.get(k)
    }
    pub fn drain(&mut self) -> Drain<K, PyArg> {
        self.table.drain()
    }
    pub unsafe fn from_ptr(ptr: *mut usize) -> PyDict<K, PyArg> {
        *(Box::from_raw(ptr as *mut PyDict<K, PyArg>))
    }
    pub fn as_ptr(self) -> *mut usize {
        Box::into_raw(Box::new(self)) as *mut usize
    }
}

impl<K> FromIterator<(K, PyArg)> for PyDict<K, PyArg>
    where K: PyDictKey + Eq + Hash
{
    fn from_iter<I: IntoIterator<Item = (K, PyArg)>>(iter: I) -> Self {
        let mut c = PyDict::new();
        for (k, v) in iter {
            c.insert(k, v);
        }
        c
    }
}

impl<T> Into<HashMap<T, PyArg>> for PyDict<T, PyArg>
    where T: PyDictKey + Eq + Hash
{
    fn into(self) -> HashMap<T, PyArg> {
        self.table
    }
}

impl<T> From<HashMap<T, PyArg>> for PyDict<T, PyArg>
    where T: PyDictKey + Eq + Hash
{
    fn from(hm: HashMap<T, PyArg>) -> PyDict<T, PyArg> {
        PyDict {
            table: hm
        }
    }
}

/// Types allowed as PyDict key values.
pub enum PyDictK {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    PyBool,
    PyString,
}

#[no_mangle]
pub extern "C" fn pydict_get_key_type(k: u32) -> *mut PyDictK {
    match k {
        1 => Box::into_raw(Box::new(PyDictK::U8)),
        2 => Box::into_raw(Box::new(PyDictK::I8)),
        3 => Box::into_raw(Box::new(PyDictK::I16)),
        4 => Box::into_raw(Box::new(PyDictK::U16)),
        5 => Box::into_raw(Box::new(PyDictK::I32)),
        6 => Box::into_raw(Box::new(PyDictK::U32)),
        7 => Box::into_raw(Box::new(PyDictK::I64)),
        8 => Box::into_raw(Box::new(PyDictK::U64)),
        11 => Box::into_raw(Box::new(PyDictK::PyBool)),
        12 => Box::into_raw(Box::new(PyDictK::PyString)),
        _ => _rustypy_abort_xtract_fail!("type not supported as PyDict key type"),
    }
}

mod key_bound {
    use pytypes::{PyString, PyBool, PyDict, PyArg};
    pub trait PyDictKey {}
    impl PyDictKey for i64 {}
    impl PyDictKey for i32 {}
    impl PyDictKey for i16 {}
    impl PyDictKey for i8 {}
    impl PyDictKey for u64 {}
    impl PyDictKey for u32 {}
    impl PyDictKey for u16 {}
    impl PyDictKey for u8 {}
    impl PyDictKey for PyString {}
    impl PyDictKey for PyBool {}

    use std::hash::Hash;
    impl<T> From<PyDict<T, PyArg>> for PyArg
        where T: Eq + Hash + PyDictKey
    {
        fn from(a: PyDict<T, PyArg>) -> PyArg {
            PyArg::PyDict(a.as_ptr())
        }
    }
}

#[no_mangle]
pub extern "C" fn pydict_new(k_type: &PyDictK) -> *mut size_t {
    match *(k_type) {
        PyDictK::I8 => {
            let d: PyDict<i8, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::I16 => {
            let d: PyDict<i16, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::I32 => {
            let d: PyDict<i32, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::I64 => {
            let d: PyDict<i64, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::U8 => {
            let d: PyDict<u8, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::U16 => {
            let d: PyDict<u16, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::U32 => {
            let d: PyDict<u32, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::U64 => {
            let d: PyDict<u64, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::PyString => {
            let d: PyDict<PyString, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
        PyDictK::PyBool => {
            let d: PyDict<PyBool, PyArg> = PyDict::new();
            d.as_ptr() as *mut size_t
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pydict_insert(dict: *mut size_t,
                                       k_type: &PyDictK,
                                       key: *mut PyArg,
                                       value: *mut PyArg) {
    macro_rules! _match_pyarg_in {
       ($p:ident; $v:tt) => {{
           match *(Box::from_raw($p as *mut PyArg)) {
               PyArg::$v(val) => { val },
               _ => _rustypy_abort_xtract_fail!("expected different key type for PyDict while inserting"),
           }
       }};
    }
    match *(k_type) {
        PyDictK::I8 => {
            let mut dict = &mut *(dict as *mut PyDict<i8, PyArg>);
            let key = _match_pyarg_in!(key; I8);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I16 => {
            let mut dict = &mut *(dict as *mut PyDict<i16, PyArg>);
            let key = _match_pyarg_in!(key; I16);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I32 => {
            let mut dict = &mut *(dict as *mut PyDict<i32, PyArg>);
            let key = _match_pyarg_in!(key; I32);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I64 => {
            let mut dict = &mut *(dict as *mut PyDict<i64, PyArg>);
            let key = _match_pyarg_in!(key; I64);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U8 => {
            let mut dict = &mut *(dict as *mut PyDict<u8, PyArg>);
            let key = _match_pyarg_in!(key; U8);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U16 => {
            let mut dict = &mut *(dict as *mut PyDict<u16, PyArg>);
            let key = _match_pyarg_in!(key; U16);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U32 => {
            let mut dict = &mut *(dict as *mut PyDict<u32, PyArg>);
            let key = _match_pyarg_in!(key; U32);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U64 => {
            let mut dict = &mut *(dict as *mut PyDict<u64, PyArg>);
            let key = _match_pyarg_in!(key; U64);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::PyString => {
            let mut dict = &mut *(dict as *mut PyDict<PyString, PyArg>);
            let key = _match_pyarg_in!(key; PyString);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::PyBool => {
            let mut dict = &mut *(dict as *mut PyDict<PyBool, PyArg>);
            let key = _match_pyarg_in!(key; PyBool);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
    };
}

#[test]
fn drain_dict() {
    unsafe {
        let mut hm = HashMap::new();
        hm.insert(0u16, PyArg::PyString(PyString::from("zero")));
        hm.insert(1u16, PyArg::PyString(PyString::from("one")));
        let dict = PyDict::from_iter(hm).as_ptr() as *mut size_t;

        let k_type = PyDictK::U16;
        let iter = pydict_get_drain(dict, &k_type);


        let e1 = pydict_drain_element(iter, &k_type);
        assert!(!e1.is_null());
        let e1: &PyTuple = &*(e1 as *const PyTuple);
        let v = match e1.next {
            Some(ref v) => &(v.elem),
            _ => panic!(),
        };
        if e1.elem == PyArg::U16(0) {
            assert_eq!(v, &PyArg::PyString(PyString::from("zero")));
        } else {
            assert_eq!(v, &PyArg::PyString(PyString::from("one")));
        }

        let e2 = pydict_drain_element(iter, &k_type);
        assert!(!e2.is_null());
        let e2: &PyTuple = &*(e2 as *const PyTuple);
        let v = match e2.next {
            Some(ref v) => &(v.elem),
            _ => panic!(),
        };
        if e2.elem == PyArg::U16(0) {
            assert_eq!(v, &PyArg::PyString(PyString::from("zero")));
        } else {
            assert_eq!(v, &PyArg::PyString(PyString::from("one")));
        }

        let e3 = pydict_drain_element(iter, &k_type);
        assert!(e3.is_null());
    }
}

#[no_mangle]
pub unsafe extern "C" fn pydict_get_drain(dict: *mut size_t, k_type: &PyDictK) -> *mut size_t {
    match *(k_type) {
        PyDictK::I8 => {
            let mut dict = &mut *(dict as *mut PyDict<i8, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::I16 => {
            let mut dict = &mut *(dict as *mut PyDict<i16, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::I32 => {
            let mut dict = &mut *(dict as *mut PyDict<i32, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::I64 => {
            let mut dict = &mut *(dict as *mut PyDict<i64, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U8 => {
            let mut dict = &mut *(dict as *mut PyDict<u8, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U16 => {
            let mut dict = &mut *(dict as *mut PyDict<u16, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U32 => {
            let mut dict = &mut *(dict as *mut PyDict<u32, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U64 => {
            let mut dict = &mut *(dict as *mut PyDict<u64, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::PyString => {
            let mut dict = &mut *(dict as *mut PyDict<PyString, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::PyBool => {
            let mut dict = &mut *(dict as *mut PyDict<PyBool, PyArg>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
    }
}

fn kv_return_tuple(k: PyArg, v: PyArg) -> *mut PyTuple {
    let ret = PyTuple {
        elem: k,
        idx: 0_usize,
        next: Some(Box::new(PyTuple {
            elem: v,
            idx: 1_usize,
            next: None,
        })),
    };
    Box::into_raw(Box::new(ret))
}

#[no_mangle]
pub unsafe extern "C" fn pydict_drain_element(iter: *mut size_t, k_type: &PyDictK) -> *mut PyTuple {
    fn _get_null() -> *mut PyTuple {
        let p: *const PyTuple = ptr::null();
        p as *mut PyTuple
    }
    match *(k_type) {
        PyDictK::I8 => {
            let mut iter = &mut *(iter as *mut Drain<i8, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::I8(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::I16 => {
            let mut iter = &mut *(iter as *mut Drain<i16, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::I16(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::I32 => {
            let mut iter = &mut *(iter as *mut Drain<i32, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::I32(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::I64 => {
            let mut iter = &mut *(iter as *mut Drain<i64, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::I64(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U8 => {
            let mut iter = &mut *(iter as *mut Drain<u8, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::U8(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U16 => {
            let mut iter = &mut *(iter as *mut Drain<u16, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::U16(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U32 => {
            let mut iter = &mut *(iter as *mut Drain<u32, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::U32(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U64 => {
            let mut iter = &mut *(iter as *mut Drain<u64, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::U64(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::PyString => {
            let mut iter = &mut *(iter as *mut Drain<PyString, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::PyString(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::PyBool => {
            let mut iter = &mut *(iter as *mut Drain<PyBool, PyArg>);
            match iter.next() {
                Some(val) => kv_return_tuple(PyArg::PyBool(val.0), val.1),
                None => _get_null(),
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pydict_get_element(dict: *mut size_t,
                                            k_type: &PyDictK,
                                            key: *mut size_t)
                                            -> *mut size_t {
    macro_rules! _match_pyarg_out {
        ($p:ident) => {{
            fn _get_null() -> *mut PyArg {
                let p: *const PyArg = ptr::null();
                p as *mut PyArg
            }
            match $p {
                PyArg::I64(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::I32(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::I16(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::I8(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::U32(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::U16(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::U8(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::F32(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::F64(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::PyBool(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::PyString(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
                PyArg::PyTuple(val) => { Box::into_raw(val) as *mut size_t },
                PyArg::PyList(val) => { Box::into_raw(val) as *mut size_t },
                _ => { _get_null() as *mut size_t },
            }
        }};
    }
    fn _get_null() -> *mut PyArg {
        let p: *const PyArg = ptr::null();
        p as *mut PyArg
    };
    match *(k_type) {
        PyDictK::I8 => {
            let mut dict = &mut *(dict as *mut PyDict<i8, PyArg>);
            let key = *(Box::from_raw(key as *mut i8));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::I16 => {
            let mut dict = &mut *(dict as *mut PyDict<i16, PyArg>);
            let key = *(Box::from_raw(key as *mut i16));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::I32 => {
            let mut dict = &mut *(dict as *mut PyDict<i32, PyArg>);
            let key = *(Box::from_raw(key as *mut i32));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::I64 => {
            let mut dict = &mut *(dict as *mut PyDict<i64, PyArg>);
            let key = *(Box::from_raw(key as *mut i64));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U8 => {
            let mut dict = &mut *(dict as *mut PyDict<u8, PyArg>);
            let key = *(Box::from_raw(key as *mut u8));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U16 => {
            let mut dict = &mut *(dict as *mut PyDict<u16, PyArg>);
            let key = *(Box::from_raw(key as *mut u16));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U32 => {
            let mut dict = &mut *(dict as *mut PyDict<u32, PyArg>);
            let key = *(Box::from_raw(key as *mut u32));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U64 => {
            let mut dict = &mut *(dict as *mut PyDict<u64, PyArg>);
            let key = *(Box::from_raw(key as *mut u64));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::PyString => {
            let mut dict = &mut *(dict as *mut PyDict<PyString, PyArg>);
            let key = *(Box::from_raw(key as *mut PyString));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::PyBool => {
            let mut dict = &mut *(dict as *mut PyDict<PyBool, PyArg>);
            let key = *(Box::from_raw(key as *mut PyBool));
            match dict.get(&key) {
                Some(ref val) => {
                    let v = (*val).clone();
                    _match_pyarg_out!(v)
                }
                None => _get_null() as *mut size_t,
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pydict_free(dict: *mut size_t, k_type: &PyDictK) {
    if dict.is_null() {
        return;
    }
    match *(k_type) {
        PyDictK::I8 => {
            Box::from_raw(dict as *mut PyDict<i8, PyArg>);
        }
        PyDictK::I16 => {
            Box::from_raw(dict as *mut PyDict<i16, PyArg>);
        }
        PyDictK::I32 => {
            Box::from_raw(dict as *mut PyDict<i32, PyArg>);
        }
        PyDictK::I64 => {
            Box::from_raw(dict as *mut PyDict<i64, PyArg>);
        }
        PyDictK::U8 => {
            Box::from_raw(dict as *mut PyDict<u8, PyArg>);
        }
        PyDictK::U16 => {
            Box::from_raw(dict as *mut PyDict<u16, PyArg>);
        }
        PyDictK::U32 => {
            Box::from_raw(dict as *mut PyDict<u16, PyArg>);
        }
        PyDictK::U64 => {
            Box::from_raw(dict as *mut PyDict<u16, PyArg>);
        }
        PyDictK::PyString => {
            Box::from_raw(dict as *mut PyDict<PyString, PyArg>);
        }
        PyDictK::PyBool => {
            Box::from_raw(dict as *mut PyDict<PyBool, PyArg>);
        }
    }
}
