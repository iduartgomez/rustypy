//! An analog of a Python dict which contains pairs of (key, values) each of a single type,
//! will accept an undefined number of one (and just one) of any other supported type
//! (including other PyDict) as value and a corresponding key of a single supported hashable
//! type (check [PyDictK](/enum.PyDictK.html to see which ones are supported)).
//!
//! PyDict can be constructed from other iterable types as long as the inner type is
//! supported (a copy will be performed in case is necessary).
//!
//! ```
//! # use rustypy::PyList;
//! # use std::iter::FromIterator;
//! PyList::from_iter(vec![1u32; 3]); // copied
//! PyList::from(vec![1u32; 3]); // moved
//! ```
//!
//! You can also use the typical hashmap interfaces (insert, get, remove, etc.) as long as the
//! type is supported (check [PyArg](../rustypy/pytypes/enum.PyArg.html) variants). PyDict
//! types and their content can be converted back to a HashMap or into a (K, V) iterator.
//!
//! ```
//! # use rustypy::PyDict;
//! # use std::collections::HashMap;
//! use std::iter::FromIterator;
//! let hm = HashMap::from_iter(vec![(0u32, "Hello"), (1, "from"), (3, "Rust")]);
//!
//! let mut d = PyDict::from(hm);
//! let hm_from_pd = PyDict::into_hashmap::<String>(d.clone());
//! assert_eq!(hm_from_pd.get(&0).unwrap(), "Hello");
//!
//! let hm_from_iter: HashMap<u32, String> = HashMap::from_iter(d.into_iter::<String>());
//! assert_eq!(hm_from_pd.get(&3).unwrap(), "Rust");
//! ```
//!
//! When extracting in Python with the FFI, elements are moved, not copied
//! and when free'd all the original elements are dropped.
//!
//! ## Unpacking PyDict from Python
//! Is recommended to use the [unpack_pydict!](../../macro.unpack_pydict!.html) macro in order
//! to convert a PyDict to a Rust native type. Check the macro documentation for more info.

use libc::size_t;
use super::PyArg;
use super::pybool::PyBool;
use super::pystring::PyString;
use super::pytuple::PyTuple;

use std::collections::HashMap;
use std::collections::hash_map::Drain;
use std::marker::PhantomData;
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

impl<K> PyDict<K,PyArg>
    where K: Eq + Hash + PyDictKey
{
    /// Creates an empty PyDict.
    pub fn new() -> PyDict<K, PyArg> {
        PyDict { table: HashMap::new() }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though; this matters for types that can be == without being
    /// identical. See the module-level documentation for more.
    pub fn insert(&mut self, k: K, v: PyArg) -> Option<PyArg> {
        self.table.insert(k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but Hash and Eq on the borrowed
    /// form must match those for the key type.
    pub fn remove(&mut self, k: &K) -> Option<PyArg> {
        self.table.remove(k)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but Hash and Eq on the borrowed
    /// form must match those for the key type.
    pub fn get(&mut self, k: &K) -> Option<&PyArg> {
        self.table.get(k)
    }

    /// Clears the map, returning all key-value pairs as an iterator.
    /// Keeps the allocated memory for reuse.
    pub fn drain(&mut self) -> Drain<K, PyArg> {
        self.table.drain()
    }

    /// Get a PyDict from a previously boxed PyDict.
    ///
    /// Takes two type parameters:
    ///     - one corresponding to the key type (check
    ///       [PyArg](../rustypy/pytypes/enum.PyArg.html) variants)
    ///     - and PyArg, corresponding to the value type
    ///
    /// And a PyDict as a raw *mut usize pointer.
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use rustypy::{PyDict, PyArg};
    /// # use std::iter::FromIterator;
    /// # let hm = HashMap::from_iter(vec![(0_u64, "some"), (1, "string")]);
    /// # let dict = PyDict::from(hm).as_ptr();
    /// let dict = unsafe { PyDict::<u64, PyArg>::from_ptr(dict) };
    /// ```
    ///
    pub unsafe fn from_ptr(ptr: *mut usize) -> PyDict<K, PyArg> {
        *(Box::from_raw(ptr as *mut PyDict<K, PyArg>))
    }

    /// Returns self as raw pointer. Use this method when returning a PyTuple to Python.
    pub fn as_ptr(self) -> *mut usize {
        Box::into_raw(Box::new(self)) as *mut usize
    }

    /// Consumes self and returns a HashMap, takes one type parameter and transforms inner
    /// content to that type.
    pub fn into_hashmap<V>(mut self) -> HashMap<K, V>
        where V: From<PyArg>
    {
        HashMap::from_iter(self.table.drain().map(|(k, v)| (k, <V>::from(v))))
    }
    /// Consume self and turn it into an iterator.
    pub fn into_iter<T: From<PyArg>>(self) -> IntoIter<K, T> {
        IntoIter {
            inner: self.table.into_iter(),
            target_t: PhantomData,
        }
    }
}

impl<K, V> FromIterator<(K, V)> for PyDict<K, PyArg>
    where K: PyDictKey + Eq + Hash,
          PyArg: From<V>
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut c = PyDict::new();
        for (k, v) in iter {
            c.insert(k, PyArg::from(v));
        }
        c
    }
}

pub struct IntoIter<K, V> {
    target_t: PhantomData<V>,
    inner: ::std::collections::hash_map::IntoIter<K, PyArg>,
}

impl<K, V> Iterator for IntoIter<K, V>
    where V: From<PyArg>,
          K: PyDictKey + Eq + Hash
{
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        match self.inner.next() {
            Some((k, v)) => Some((k, <V>::from(v))),
            None => None,
        }
    }
    fn collect<B>(self) -> B
        where B: FromIterator<Self::Item>
    {
        self.inner.map(|(k, v)| (k, <V>::from(v))).collect::<B>()
    }
}

impl<K, V> From<HashMap<K, V>> for PyDict<K, PyArg>
    where K: PyDictKey + Eq + Hash,
          PyArg: From<V>
{
    fn from(mut hm: HashMap<K, V>) -> PyDict<K, PyArg> {
        PyDict {
            table: hm.drain().map(|(k, v)| (k, PyArg::from(v))).collect::<HashMap<K, PyArg>>(),
        }
    }
}

impl<K> From<PyDict<K, PyArg>> for PyArg
    where K: Eq + Hash + PyDictKey
{
    fn from(a: PyDict<K, PyArg>) -> PyArg {
        PyArg::PyDict(a.as_ptr())
    }
}

impl<K> From<PyArg> for PyDict<K, PyArg>
    where K: Eq + Hash + PyDictKey
{
    fn from(a: PyArg) -> PyDict<K, PyArg> {
        match a {
            PyArg::PyDict(v) => unsafe { *(Box::from_raw(v as *mut PyDict<K, PyArg>)) },
            _ => _rustypy_abort_xtract_fail!("expected a PyDict while destructuring PyArg enum"),
        }
    }
}

impl<K, V> From<HashMap<K, V>> for PyArg
    where PyArg: From<V>,
          K: Eq + Hash + PyDictKey
{
    fn from(a: HashMap<K, V>) -> PyArg {
        let dict = PyDict::from(a);
        PyArg::PyDict(dict.as_ptr())
    }
}

/// Consumes a `*mut PyDict<K, PyArg<T>> as *mut usize` content and returns a `HashMap<K, T>`
/// from it, no copies are performed in the process.
///
/// All inner elements are moved out if possible, if not (like with PyTuples) are copied.
/// PyTuple variants are destructured into Rust tuples which contain the appropiate Rust types
/// (valid syntax for [unpack_pytuple!](../rustypy/macro.unpack_pytuple!.html) macro must
/// be provided). The same happens with other container types (inner PyList, PyDict, etc.).
///
/// # Examples
///
/// A simple PyDict with i32 keys which contains PyString types::
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// use rustypy::{PyDict, PyString};
/// use std::collections::HashMap;
///
/// let mut hm = HashMap::new();
/// for (k, v) in vec![(0_i32, "Hello"), (1_i32, " "), (2_i32, "World!")] {
///     hm.insert(k, v);
/// }
/// let dict = PyDict::from(hm).as_ptr();
/// let unpacked = unpack_pydict!(dict; PyDict{(i32, PyString => String)});
/// # }
/// ```
///
/// With nested container types:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use rustypy::{PyDict, PyString, PyArg};
/// # use std::collections::HashMap;
/// # let mut hm0 = HashMap::new();
/// # for (k, v) in vec![(0_i32, "Hello"), (1, " "), (2, "World!")] {
/// #     hm0.insert(k, v);
/// # }
/// # let mut hm1 = HashMap::new();
/// # for i in 0..3_i64 {
/// #     let k = format!("#{}", i);
/// #     let v = PyArg::from(hm0.clone());
/// #     let l = PyArg::from(vec![0_i64; 3]);
/// #     hm1.insert(k, pytuple!(l, v));
/// # }
/// # let dict = PyDict::from(hm0).as_ptr();
/// // dict from Python: {str: ([u64], {i32: str})} as *mut usize
/// let unpacked = unpack_pydict!(dict;
///     PyDict{(PyString, PyTuple{({PyList{I64 => i64}}, {PyDict{(i32, PyString => String)}},)})} );
/// # }
/// ```
///
#[macro_export]
macro_rules! unpack_pydict {
    ( $pydict:ident; PyDict{($kt:ty, $o:tt { $($t:tt)* })} ) => {{
        use rustypy::{PyArg, PyDict};
        let mut unboxed = unsafe { *(Box::from_raw($pydict as *mut PyDict<$kt, PyArg>)) };
        use std::collections::HashMap;
        let mut dict = HashMap::new();
        for (k, v) in unboxed.drain() {
            match v {
                PyArg::$o(val) => {
                    let inner = unpack_pydict!(val; $o { $($t)* });
                    dict.insert(k, inner);
                }
                _ => _rustypy_abort_xtract_fail!("failed while converting PyDict to HashMap")
            }
        }
        dict
    }};
    ( $pytuple:ident; PyTuple { $t:tt } ) => {{
        let mut unboxed = *($pytuple);
        unpack_pytuple!(unboxed; $t)
    }};
    ( $pylist:ident; PyList{ $($u:tt)* } ) => {{
        unpack_pylist!( $pylist; PyList{ $($u)* } )
    }};
    ( $pydict:ident; PyDict{($kt:ty, $t:tt => $type_:ty)} ) => {{
        use rustypy::{PyArg, PyDict};
        let mut unboxed = unsafe { *(Box::from_raw($pydict as *mut PyDict<$kt, PyArg>)) };
        use std::collections::HashMap;
        let mut dict = HashMap::new();
        for (k, v) in unboxed.drain() {
            match v {
                PyArg::$t(val) => { dict.insert(k, <$type_>::from(val)); },
                _ => _rustypy_abort_xtract_fail!("failed while converting PyDict to HashMap"),
            }
        }
        dict
    }};
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
               _ => _rustypy_abort_xtract_fail!("expected different key type \
                                                for PyDict while inserting a (key, val) pair"),
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

mod key_bound {
    use pytypes::pystring::PyString;
    use pytypes::pybool::PyBool;

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
