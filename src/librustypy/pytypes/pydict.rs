//! An analog of a Python dict which contains pairs of (key, values) each of a single type,
//! will accept an undefined number of one (and just one) of any other supported type
//! (including other PyDict) as value and a corresponding key of a single supported hashable
//! type (check [PyDictK](../pydict/enum.PyDictK.html) to see which ones are supported).
//!
//! PyDict can be constructed from other iterable types as long as the inner type is
//! supported (a copy will be performed in case is necessary).
//!
//! ```
//! # use rustypy::PyDict;
//! # use std::collections::HashMap;
//! use std::iter::FromIterator;
//! let mut hm = HashMap::from_iter(vec![(0u32, "Hi"), (1, "Rust")]);
//! PyDict::from_iter(hm.clone().into_iter());
//! PyDict::from(hm);
//! ```
//!
//! You can also use the typical hashmap interfaces (insert, get, remove, etc.) as long as the
//! type is supported (check [PyArg](../rustypy/pytypes/enum.PyArg.html) variants). PyDict
//! types and their content can be converted back to a HashMap or into a (K, V) iterator.
//!
//! ```
//! # use rustypy::{PyDict, PyArg};
//! # use std::collections::HashMap;
//! use std::iter::FromIterator;
//! let hm = HashMap::from_iter(vec![(0u32, "Hello"), (1, "from"), (3, "Rust")]);
//!
//! let ptr = PyDict::from(hm).into_raw();
//! // to get a PyDict from a raw pointer we need to provide the key type:
//! let d = unsafe { PyDict::<u32>::from_ptr(ptr) };
//! let hm_from_pd = PyDict::into_hashmap::<String>(d.clone());
//! assert_eq!(hm_from_pd.get(&0).unwrap(), "Hello");
//!
//! let hm_from_iter: HashMap<u32, String> = HashMap::from_iter(d.speciallized_iter::<String>());
//! assert_eq!(hm_from_pd.get(&3).unwrap(), "Rust");
//! ```
//!
//! When extracting in Python with the FFI, elements are moved, not copied
//! and when free'd all the original elements are dropped.
//!
//! # Safety
//! PyList must be passed between Rust and Python as a ```size_t``` raw pointer. You can get a
//! raw pointer using ```into_raw``` and convert from a raw pointer using the "static"
//! method ```PyDict::from_ptr``` which is unsafe as it requires dereferencing a raw pointer.
//! PyDict also require providing the key type, in case the key type is not the expected one
//! undefined behaviour will happen.
//!
//! For convinience there are some methods to perform conversions to ```HashMap<K,V>```
//! from ```PyDict<K,PyArg>```, while none of those are unsafe per se,
//! they require providing the expected PyArg enum variant.
//! In case the expected variant is wrong, the process will abort and exit as it's not possible
//! to handle errors acrosss the FFI boundary.
//!
//! ## Unpacking PyDict from Python
//! Is recommended to use the [unpack_pydict!](../../macro.unpack_pydict!.html) macro in order
//! to convert a PyDict to a Rust native type. Check the macro documentation for more info.

use super::{abort_and_exit, PyArg, PyBool, PyList, PyString, PyTuple};
use libc::size_t;

use std::collections::hash_map::Drain;
use std::collections::HashMap;
use std::convert::AsRef;
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PyDict<K>
where
    K: Eq + Hash + PyDictKey,
{
    _inner: HashMap<K, PyArg>,
}

pub(crate) use self::key_bound::PyDictKey;

impl<K> PyDict<K>
where
    K: Eq + Hash + PyDictKey,
{
    /// Creates an empty PyDict.
    pub fn new() -> PyDict<K> {
        PyDict {
            _inner: HashMap::new(),
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though; this matters for types that can be == without being
    /// identical. See the module-level documentation for more.
    pub fn insert<V>(&mut self, k: K, v: V) -> Option<V>
    where
        PyArg: From<V>,
        V: From<PyArg>,
    {
        if let Some(val) = self._inner.insert(k, PyArg::from(v)) {
            Some(V::from(val))
        } else {
            None
        }
    }

    /// Removes a key from the map, returning the value at the key if the key was previously
    /// in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but Hash and Eq on the borrowed
    /// form must match those for the key type.
    pub fn remove<V>(&mut self, k: &K) -> Option<V>
    where
        V: From<PyArg>,
    {
        if let Some(val) = self._inner.remove(k) {
            Some(V::from(val))
        } else {
            None
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but Hash and Eq on the borrowed
    /// form must match those for the key type.
    pub fn get<'a, V>(&'a self, k: &K) -> Option<&'a V>
    where
        PyArg: AsRef<V>,
    {
        if let Some(rval) = self._inner.get(k) {
            Some(rval.as_ref())
        } else {
            None
        }
    }

    fn get_mut_pyarg(&mut self, k: &K) -> Option<&mut PyArg> {
        self._inner.get_mut(k)
    }

    /// Clears the map, returning all key-value pairs as an iterator.
    /// Keeps the allocated memory for reuse.
    #[doc(hidden)]
    pub fn drain(&mut self) -> Drain<K, PyArg> {
        self._inner.drain()
    }

    /// Get a PyDict from a previously boxed PyDict.
    ///
    /// Takes the key as type parameter `K`, the raw pointer to the dictionary as argument
    /// and returns a PyDict with key type `K` (check
    /// [PyArg](../rustypy/pytypes/pydict/enum.PyDictK.html) variants for allowed key types).
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use rustypy::{PyDict, PyArg};
    /// # use std::iter::FromIterator;
    /// # let hm = HashMap::from_iter(vec![(0_u64, "some"), (1, "string")]);
    /// # let dict = PyDict::from(hm).into_raw();
    /// let dict = unsafe { PyDict::<u64>::from_ptr(dict) };
    /// ```
    ///
    pub unsafe fn from_ptr(ptr: *mut size_t) -> PyDict<K> {
        *(Box::from_raw(ptr as *mut PyDict<K>))
    }

    /// Returns self as raw pointer. Use this method when returning a PyDict to Python.
    pub fn into_raw(self) -> *mut size_t {
        Box::into_raw(Box::new(self)) as *mut size_t
    }

    /// Consumes self and returns a HashMap, takes one type parameter and transforms inner
    /// content to that type.
    pub fn into_hashmap<V>(mut self) -> HashMap<K, V>
    where
        V: From<PyArg>,
    {
        HashMap::from_iter(self._inner.drain().map(|(k, v)| (k, <V>::from(v))))
    }

    /// Consume self and turn it into an iterator.
    pub fn speciallized_iter<V: From<PyArg>>(self) -> IntoIter<K, V> {
        IntoIter {
            inner: self._inner.into_iter(),
            target_t: PhantomData,
        }
    }
}

impl<K, V> FromIterator<(K, V)> for PyDict<K>
where
    K: PyDictKey + Eq + Hash,
    PyArg: From<V>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut hm = HashMap::new();
        for (k, v) in iter.into_iter() {
            hm.insert(k, PyArg::from(v));
        }
        PyDict { _inner: hm }
    }
}

impl<K, V> From<HashMap<K, V>> for PyDict<K>
where
    K: PyDictKey + Eq + Hash,
    PyArg: From<V>,
{
    fn from(mut hm: HashMap<K, V>) -> PyDict<K> {
        PyDict {
            _inner: hm
                .drain()
                .map(|(k, v)| (k, PyArg::from(v)))
                .collect::<HashMap<K, PyArg>>(),
        }
    }
}

pub struct IntoIter<K, V> {
    target_t: PhantomData<V>,
    inner: ::std::collections::hash_map::IntoIter<K, PyArg>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    V: From<PyArg>,
    K: PyDictKey + Eq + Hash,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        match self.inner.next() {
            Some((k, v)) => Some((k, <V>::from(v))),
            None => None,
        }
    }
    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
    {
        self.inner.map(|(k, v)| (k, <V>::from(v))).collect::<B>()
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn pydict_new(k_type: &PyDictK) -> *mut size_t {
    match *(k_type) {
        PyDictK::I8 => {
            let d: PyDict<i8> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::I16 => {
            let d: PyDict<i16> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::I32 => {
            let d: PyDict<i32> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::I64 => {
            let d: PyDict<i64> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::U8 => {
            let d: PyDict<u8> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::U16 => {
            let d: PyDict<u16> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::U32 => {
            let d: PyDict<u32> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::U64 => {
            let d: PyDict<u64> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::PyString => {
            let d: PyDict<PyString> = PyDict::new();
            d.into_raw() as *mut size_t
        }
        PyDictK::PyBool => {
            let d: PyDict<PyBool> = PyDict::new();
            d.into_raw() as *mut size_t
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_insert(
    dict: *mut size_t,
    k_type: &PyDictK,
    key: *mut PyArg,
    value: *mut PyArg,
) {
    macro_rules! _match_pyarg_in {
        ($p:ident; $v:tt) => {{
            match *(Box::from_raw($p as *mut PyArg)) {
                PyArg::$v(val) => val,
                _ => abort_and_exit(
                    "expected different key type \
                     for PyDict while inserting a (key, val) pair",
                ),
            }
        }};
    }
    match *(k_type) {
        PyDictK::I8 => {
            let dict = &mut *(dict as *mut PyDict<i8>);
            let key = _match_pyarg_in!(key; I8);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I16 => {
            let dict = &mut *(dict as *mut PyDict<i16>);
            let key = _match_pyarg_in!(key; I16);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I32 => {
            let dict = &mut *(dict as *mut PyDict<i32>);
            let key = _match_pyarg_in!(key; I32);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::I64 => {
            let dict = &mut *(dict as *mut PyDict<i64>);
            let key = _match_pyarg_in!(key; I64);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U8 => {
            let dict = &mut *(dict as *mut PyDict<u8>);
            let key = _match_pyarg_in!(key; U8);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U16 => {
            let dict = &mut *(dict as *mut PyDict<u16>);
            let key = _match_pyarg_in!(key; U16);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U32 => {
            let dict = &mut *(dict as *mut PyDict<u32>);
            let key = _match_pyarg_in!(key; U32);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::U64 => {
            let dict = &mut *(dict as *mut PyDict<u64>);
            let key = _match_pyarg_in!(key; U64);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::PyString => {
            let dict = &mut *(dict as *mut PyDict<PyString>);
            let key = _match_pyarg_in!(key; PyString);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
        PyDictK::PyBool => {
            let dict = &mut *(dict as *mut PyDict<PyBool>);
            let key = _match_pyarg_in!(key; PyBool);
            let value = *(Box::from_raw(value));
            dict.insert(key, value);
        }
    };
}

#[test]
fn drain_dict() {
    unsafe {
        let match_kv = |kv: *mut PyDictPair| match *Box::from_raw(kv as *mut PyDictPair) {
            PyDictPair {
                key: PyArg::U16(0),
                val: PyArg::PyString(val),
            } => {
                assert_eq!(val, PyString::from("zero"));
            }
            PyDictPair {
                key: PyArg::U16(1),
                val: PyArg::PyString(val),
            } => {
                assert_eq!(val, PyString::from("one"));
            }
            _ => panic!(),
        };

        let mut hm = HashMap::new();
        hm.insert(0u16, PyArg::PyString(PyString::from("zero")));
        hm.insert(1u16, PyArg::PyString(PyString::from("one")));
        let dict = PyDict::from_iter(hm).into_raw() as *mut size_t;

        let k_type = PyDictK::U16;
        let iter = pydict_get_drain(dict, &k_type);

        let e0 = pydict_drain_element(iter, &k_type);
        assert!(!e0.is_null());
        match_kv(e0);

        let e1 = pydict_drain_element(iter, &k_type);
        assert!(!e1.is_null());
        match_kv(e1);

        let none = pydict_drain_element(iter, &k_type);
        assert!(none.is_null());
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_get_drain(dict: *mut size_t, k_type: &PyDictK) -> *mut size_t {
    match *(k_type) {
        PyDictK::I8 => {
            let dict = &mut *(dict as *mut PyDict<i8>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::I16 => {
            let dict = &mut *(dict as *mut PyDict<i16>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::I32 => {
            let dict = &mut *(dict as *mut PyDict<i32>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::I64 => {
            let dict = &mut *(dict as *mut PyDict<i64>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U8 => {
            let dict = &mut *(dict as *mut PyDict<u8>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U16 => {
            let dict = &mut *(dict as *mut PyDict<u16>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U32 => {
            let dict = &mut *(dict as *mut PyDict<u32>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::U64 => {
            let dict = &mut *(dict as *mut PyDict<u64>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::PyString => {
            let dict = &mut *(dict as *mut PyDict<PyString>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
        PyDictK::PyBool => {
            let dict = &mut *(dict as *mut PyDict<PyBool>);
            Box::into_raw(Box::new(dict.drain())) as *mut size_t
        }
    }
}

#[doc(hidden)]
pub struct PyDictPair {
    key: PyArg,
    val: PyArg,
}

impl PyDictPair {
    fn kv_return_tuple(k: PyArg, v: PyArg) -> *mut PyDictPair {
        Box::into_raw(Box::new(PyDictPair { key: k, val: v }))
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_get_kv(a: i32, pair: *mut PyDictPair) -> *mut PyArg {
    let pair = &mut *(pair);
    match a {
        0 => {
            let k = mem::replace(&mut pair.key, PyArg::None);
            Box::into_raw(Box::new(k))
        }
        1 => {
            let v = mem::replace(&mut pair.val, PyArg::None);
            Box::into_raw(Box::new(v))
        }
        _ => panic!(),
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_free_kv(pair: *mut PyDictPair) {
    Box::from_raw(pair);
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_drain_element(
    iter: *mut size_t,
    k_type: &PyDictK,
) -> *mut PyDictPair {
    fn _get_null() -> *mut PyDictPair {
        let p: *mut PyDictPair = ptr::null_mut();
        p
    }
    match *(k_type) {
        PyDictK::I8 => {
            let iter = &mut *(iter as *mut Drain<i8, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::I8(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::I16 => {
            let iter = &mut *(iter as *mut Drain<i16, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::I16(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::I32 => {
            let iter = &mut *(iter as *mut Drain<i32, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::I32(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::I64 => {
            let iter = &mut *(iter as *mut Drain<i64, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::I64(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U8 => {
            let iter = &mut *(iter as *mut Drain<u8, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::U8(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U16 => {
            let iter = &mut *(iter as *mut Drain<u16, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::U16(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U32 => {
            let iter = &mut *(iter as *mut Drain<u32, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::U32(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::U64 => {
            let iter = &mut *(iter as *mut Drain<u64, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::U64(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::PyString => {
            let iter = &mut *(iter as *mut Drain<PyString, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::PyString(val.0), val.1),
                None => _get_null(),
            }
        }
        PyDictK::PyBool => {
            let iter = &mut *(iter as *mut Drain<PyBool, PyArg>);
            match iter.next() {
                Some(val) => PyDictPair::kv_return_tuple(PyArg::PyBool(val.0), val.1),
                None => _get_null(),
            }
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_get_mut_element(
    dict: *mut size_t,
    k_type: &PyDictK,
    key: *mut size_t,
) -> *mut size_t {
    macro_rules! _match_pyarg_out {
        ($p:ident) => {{
            match *$p {
                PyArg::I64(ref mut val) => val as *mut i64 as *mut size_t,
                PyArg::I32(ref mut val) => val as *mut i32 as *mut size_t,
                PyArg::I16(ref mut val) => val as *mut i16 as *mut size_t,
                PyArg::I8(ref mut val) => val as *mut i8 as *mut size_t,
                PyArg::U32(ref mut val) => val as *mut u32 as *mut size_t,
                PyArg::U16(ref mut val) => val as *mut u16 as *mut size_t,
                PyArg::U8(ref mut val) => val as *mut u8 as *mut size_t,
                PyArg::F32(ref mut val) => val as *mut f32 as *mut size_t,
                PyArg::F64(ref mut val) => val as *mut f64 as *mut size_t,
                PyArg::PyBool(ref mut val) => val as *mut PyBool as *mut size_t,
                PyArg::PyString(ref mut val) => val as *mut PyString as *mut size_t,
                PyArg::PyTuple(ref mut val) => &mut **val as *mut PyTuple as *mut size_t,
                PyArg::PyList(ref mut val) => &mut **val as *mut PyList as *mut size_t,
                PyArg::PyDict(rval) => rval,
                _ => _get_null() as *mut size_t,
            }
        }};
    }
    fn _get_null() -> *mut PyArg {
        let p: *mut PyArg = ptr::null_mut();
        p
    };
    match *(k_type) {
        PyDictK::I8 => {
            let dict = &mut *(dict as *mut PyDict<i8>);
            let key = *(Box::from_raw(key as *mut i8));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::I16 => {
            let dict = &mut *(dict as *mut PyDict<i16>);
            let key = *(Box::from_raw(key as *mut i16));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::I32 => {
            let dict = &mut *(dict as *mut PyDict<i32>);
            let key = *(Box::from_raw(key as *mut i32));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::I64 => {
            let dict = &mut *(dict as *mut PyDict<i64>);
            let key = *(Box::from_raw(key as *mut i64));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U8 => {
            let dict = &mut *(dict as *mut PyDict<u8>);
            let key = *(Box::from_raw(key as *mut u8));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U16 => {
            let dict = &mut *(dict as *mut PyDict<u16>);
            let key = *(Box::from_raw(key as *mut u16));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U32 => {
            let dict = &mut *(dict as *mut PyDict<u32>);
            let key = *(Box::from_raw(key as *mut u32));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::U64 => {
            let dict = &mut *(dict as *mut PyDict<u64>);
            let key = *(Box::from_raw(key as *mut u64));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::PyString => {
            let dict = &mut *(dict as *mut PyDict<PyString>);
            let key = *(Box::from_raw(key as *mut PyString));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
        PyDictK::PyBool => {
            let dict = &mut *(dict as *mut PyDict<PyBool>);
            let key = *(Box::from_raw(key as *mut PyBool));
            match dict.get_mut_pyarg(&key) {
                Some(val) => _match_pyarg_out!(val),
                None => _get_null() as *mut size_t,
            }
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn pydict_free(dict: *mut size_t, k_type: &PyDictK) {
    if dict.is_null() {
        return;
    }
    match *(k_type) {
        PyDictK::I8 => {
            Box::from_raw(dict as *mut PyDict<i8>);
        }
        PyDictK::I16 => {
            Box::from_raw(dict as *mut PyDict<i16>);
        }
        PyDictK::I32 => {
            Box::from_raw(dict as *mut PyDict<i32>);
        }
        PyDictK::I64 => {
            Box::from_raw(dict as *mut PyDict<i64>);
        }
        PyDictK::U8 => {
            Box::from_raw(dict as *mut PyDict<u8>);
        }
        PyDictK::U16 => {
            Box::from_raw(dict as *mut PyDict<u16>);
        }
        PyDictK::U32 => {
            Box::from_raw(dict as *mut PyDict<u16>);
        }
        PyDictK::U64 => {
            Box::from_raw(dict as *mut PyDict<u16>);
        }
        PyDictK::PyString => {
            Box::from_raw(dict as *mut PyDict<PyString>);
        }
        PyDictK::PyBool => {
            Box::from_raw(dict as *mut PyDict<PyBool>);
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

pub(crate) mod key_bound {
    use crate::pytypes::pybool::PyBool;
    use crate::pytypes::pystring::PyString;

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
}

#[doc(hidden)]
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
        _ => abort_and_exit("type not supported as PyDict key type"),
    }
}
