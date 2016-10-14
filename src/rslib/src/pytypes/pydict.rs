use libc::size_t;
use pytypes::{PyArg, PyBool, PyString};

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
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
    pub fn as_ptr(self) -> *mut PyDict<K, PyArg> {
        Box::into_raw(Box::new(self))
    }
    pub unsafe fn from_ptr(ptr: *mut PyDict<K, PyArg>) -> PyDict<K, PyArg> {
        *(Box::from_raw(ptr))
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
}

pub enum PyDictK {
    I64,
    I32,
    I16,
    I8,
    U32,
    U16,
    U8,
    PyBool,
    PyString,
}

mod key_bound {
    use pytypes::{PyString, PyBool};
    pub trait PyDictKey {}
    impl PyDictKey for i64 {}
    impl PyDictKey for i32 {}
    impl PyDictKey for i16 {}
    impl PyDictKey for i8 {}
    impl PyDictKey for u32 {}
    impl PyDictKey for u16 {}
    impl PyDictKey for u8 {}
    impl PyDictKey for PyString {}
    impl PyDictKey for PyBool {}
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
                                       key: *mut size_t,
                                       value: *mut PyArg) {
    match *(k_type) {
        PyDictK::I8 => {
            let mut dict = &mut *(dict as *mut PyDict<i8, PyArg>);
            let key = *(Box::from_raw(key as *mut i8));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I16 => {
            let mut dict = &mut *(dict as *mut PyDict<i16, PyArg>);
            let key = *(Box::from_raw(key as *mut i16));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I32 => {
            let mut dict = &mut *(dict as *mut PyDict<i32, PyArg>);
            let key = *(Box::from_raw(key as *mut i32));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I64 => {
            let mut dict = &mut *(dict as *mut PyDict<i64, PyArg>);
            let key = *(Box::from_raw(key as *mut i64));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U8 => {
            let mut dict = &mut *(dict as *mut PyDict<u8, PyArg>);
            let key = *(Box::from_raw(key as *mut u8));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U16 => {
            let mut dict = &mut *(dict as *mut PyDict<u16, PyArg>);
            let key = *(Box::from_raw(key as *mut u16));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U32 => {
            let mut dict = &mut *(dict as *mut PyDict<u32, PyArg>);
            let key = *(Box::from_raw(key as *mut u32));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::PyString => {
            let mut dict = &mut *(dict as *mut PyDict<PyString, PyArg>);
            let key = *(Box::from_raw(key as *mut PyString));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::PyBool => {
            let mut dict = &mut *(dict as *mut PyDict<PyBool, PyArg>);
            let key = *(Box::from_raw(key as *mut PyBool));
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
    };
}

pub struct DictPair<T> {
    key: T,
    val: PyArg,
}

use std::ptr;

macro_rules! match_pyarg {
    ($t:ty; $p:expr) => {{
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
            PyArg::PyTuple(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
            PyArg::PyList(val) => { Box::into_raw(Box::new(val)) as *mut size_t },
            PyArg::None => {
                let p: *const $t = ptr::null();
                p as *mut size_t
            }
        }
    }};
}

pub unsafe extern "C" fn pydict_pair_get_val(k_type: &PyDictK, pair: *mut size_t) -> *mut size_t {
    match *(k_type) {
        PyDictK::I8 => {
            let pair = Box::from_raw(pair as *mut DictPair<i8>);
            match_pyarg!(i8; pair.val)
        }
        PyDictK::I16 => {
            let pair = Box::from_raw(pair as *mut DictPair<i16>);
            match_pyarg!(i16; pair.val)
        }
        PyDictK::I32 => {
            let pair = Box::from_raw(pair as *mut DictPair<i32>);
            match_pyarg!(i32; pair.val)
        }
        PyDictK::I64 => {
            let pair = Box::from_raw(pair as *mut DictPair<i64>);
            match_pyarg!(i64; pair.val)
        }
        PyDictK::U8 => {
            let pair = Box::from_raw(pair as *mut DictPair<u8>);
            match_pyarg!(u8; pair.val)
        }
        PyDictK::U16 => {
            let pair = Box::from_raw(pair as *mut DictPair<u16>);
            match_pyarg!(u16; pair.val)
        }
        PyDictK::U32 => {
            let pair = Box::from_raw(pair as *mut DictPair<u32>);
            match_pyarg!(u32; pair.val)
        }
        PyDictK::PyString => {
            let pair = Box::from_raw(pair as *mut DictPair<PyString>);
            match_pyarg!(PyString; pair.val)
        }
        PyDictK::PyBool => {
            let pair = Box::from_raw(pair as *mut DictPair<PyBool>);
            match_pyarg!(PyBool; pair.val)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pydict_get(dict: *mut size_t,
                                    k_type: &PyDictK,
                                    key: *mut size_t)
                                    -> *mut size_t {
    match *(k_type) {
        PyDictK::I8 => {
            let mut dict = &mut *(dict as *mut PyDict<i8, PyArg>);
            let key = *(Box::from_raw(key as *mut i8));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::I16 => {
            let mut dict = &mut *(dict as *mut PyDict<i16, PyArg>);
            let key = *(Box::from_raw(key as *mut i16));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::I32 => {
            let mut dict = &mut *(dict as *mut PyDict<i32, PyArg>);
            let key = *(Box::from_raw(key as *mut i32));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::I64 => {
            let mut dict = &mut *(dict as *mut PyDict<i64, PyArg>);
            let key = *(Box::from_raw(key as *mut i64));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::U8 => {
            let mut dict = &mut *(dict as *mut PyDict<u8, PyArg>);
            let key = *(Box::from_raw(key as *mut u8));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::U16 => {
            let mut dict = &mut *(dict as *mut PyDict<u16, PyArg>);
            let key = *(Box::from_raw(key as *mut u16));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::U32 => {
            let mut dict = &mut *(dict as *mut PyDict<u32, PyArg>);
            let key = *(Box::from_raw(key as *mut u32));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::PyString => {
            let mut dict = &mut *(dict as *mut PyDict<PyString, PyArg>);
            let key = *(Box::from_raw(key as *mut PyString));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
            }
        }
        PyDictK::PyBool => {
            let mut dict = &mut *(dict as *mut PyDict<PyBool, PyArg>);
            let key = *(Box::from_raw(key as *mut PyBool));
            match dict.get(&key) {
                Some(val) => {
                    Box::into_raw(Box::new(DictPair {
                        val: val.clone(),
                        key: key,
                    })) as *mut size_t
                }
                None => {
                    Box::into_raw(Box::new(DictPair {
                        val: PyArg::None,
                        key: key,
                    })) as *mut size_t
                }
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
            let dict = dict as *mut PyDict<i8, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::I16 => {
            let dict = dict as *mut PyDict<i16, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::I32 => {
            let dict = dict as *mut PyDict<i32, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::I64 => {
            let dict = dict as *mut PyDict<i64, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::U8 => {
            let dict = dict as *mut PyDict<u8, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::U16 => {
            let dict = dict as *mut PyDict<u16, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::U32 => {
            let dict = dict as *mut PyDict<u32, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::PyString => {
            let dict = dict as *mut PyDict<PyString, PyArg>;
            PyDict::from_ptr(dict);
        }
        PyDictK::PyBool => {
            let dict = dict as *mut PyDict<PyBool, PyArg>;
            PyDict::from_ptr(dict);
        }
    }
}
