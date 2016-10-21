//! An analog of a Python list which contains elements of a single type, will accept an
//! undefined number of one (and just one) of any other supported type (including other
//! PyLists).
//!
//! PyList can be constructed from other iterable types as long as the inner type is
//! supported (a copy will be performed in case is necessary).
//!
//! ```
//! # use rustypy::PyList;
//! # use std::iter::FromIterator;
//! PyList::from_iter(vec![1u32; 3]); // copied
//! PyList::from(vec![1u32; 3]); // moved
//! ```
//!
//! You can also use the typical vector interfaces (push, pop, remove, etc.) as long as the
//! type is supported (check [PyArg](../rustypy/pytypes/enum.PyArg.html) variants). PyList
//! types and their content can also be implicitly converted through an special iterator type.
//!
//! ```
//! # use rustypy::PyList;
//! let mut l = PyList::new();
//! for e in vec![0u32, 1, 3] {
//!     l.push(e);
//! }
//!
//! let mut iter = PyList::into_iter::<u32>(l);
//! assert_eq!(iter.collect::<Vec<u32>>(), vec![0u32, 1, 3])
//! ```
//!
//! When extracting in Python with the FFI, elements are moved whenever is possible, not copied
//! and when free'd all the original elements are dropped.
//!
//! ## Unpacking PyList from Python
//! Is recommended to use the [unpack_pylist!](../../macro.unpack_pylist!.html) macro in order
//! to convert a PyList to a Rust native type. Check the macro documentation for more info.

use pytypes::PyArg;

use std::ops::{Index, IndexMut};
use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;

/// An analog of a Python list which contains an undefined number of elements of
/// a single kind, of any [supported type](../../../rustypy/pytypes/enum.PyArg.html).
///
/// Read the [module docs](index.html) for more information.
#[derive(Clone, Debug, PartialEq)]
pub struct PyList {
    members: Vec<PyArg>,
}

impl PyList {
    /// Constructs a new, empty ```Vec<T>```.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    pub fn new() -> PyList {
        PyList { members: Vec::new() }
    }

    /// Removes and returns the element at position ```index``` within the vector,
    /// shifting all elements after it to the left.
    pub fn remove(&mut self, index: usize) -> PyArg {
        self.members.remove(index)
    }

    /// Removes the last element from a vector and returns it, or ```None``` if it is empty.
    pub fn pop(&mut self) -> Option<PyArg> {
        self.members.pop()
    }

    /// Returns the number of elements in the PyList.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Appends an element to the back of a collection.
    ///
    /// ##Panics
    ///
    /// Panics if the number of elements in the vector overflows a usize.
    pub fn push<T>(&mut self, a: T)
        where PyArg: From<T>
    {
        self.members.push(PyArg::from(a))
    }

    /// Get a PyList from a previously boxed raw pointer.
    pub unsafe fn from_ptr(ptr: *mut PyList) -> PyList {
        *(Box::from_raw(ptr))
    }

    /// Return a PyList as a raw pointer.
    pub fn as_ptr(self) -> *mut PyList {
        Box::into_raw(Box::new(self))
    }

    /// Consume self and turn it into an iterator.
    pub fn into_iter<T: From<PyArg>>(self) -> IntoIter<T> {
        IntoIter {
            inner: self.members.into_iter(),
            target_t: PhantomData,
        }
    }
}

impl<T> FromIterator<T> for PyList
    where PyArg: From<T>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = PyList::new();
        for e in iter {
            c.push(e);
        }
        c
    }
}

pub struct IntoIter<T> {
    target_t: PhantomData<T>,
    inner: ::std::vec::IntoIter<PyArg>,
}

impl<T> Iterator for IntoIter<T>
    where T: From<PyArg>
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.inner.next() {
            Some(val) => Some(<T>::from(val)),
            None => None,
        }
    }
    fn collect<B>(self) -> B
        where B: FromIterator<Self::Item>
    {
        self.inner.map(|x| <T>::from(x)).collect::<B>()
    }
}

impl<T> Into<Vec<T>> for PyList
    where PyArg: Into<T>
{
    fn into(mut self) -> Vec<T> {
        self.members.drain(..).map(|x| PyArg::into(x)).collect()
    }
}

impl<T> From<Vec<T>> for PyList
    where PyArg: From<T>
{
    fn from(mut v: Vec<T>) -> PyList {
        PyList { members: v.drain(..).map(|x| PyArg::from(x)).collect() }
    }
}

impl Index<usize> for PyList {
    type Output = PyArg;
    fn index(&self, index: usize) -> &PyArg {
        &(self.members[index])
    }
}

impl<'a> IndexMut<usize> for PyList {
    fn index_mut(&mut self, index: usize) -> &mut PyArg {
        &mut (self.members[index])
    }
}

/// Consumes a `Box<PyList<PyArg(T)>>` content and returns a `Vec<T>` from it, no copies
/// are performed in the process.
///
/// All inner elements are moved out if possible, if not (like with PyTuples) are copied.
/// PyTuple variants are destructured into Rust tuples which contain the appropiate Rust types
/// (valid syntax for [unpack_pytuple!](../rustypy/macro.unpack_pytuple!.html) macro must
/// be provided). The same for other container types (inner PyList, PyDict, etc.).
///
/// # Examples
///
/// A simple PyList which contains PyString types::
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// use rustypy::{PyList, PyString};
/// let string_list = Box::new(PyList::from(vec!["Python", "in", "Rust"]));
/// let unpacked = unpack_pylist!(string_list; PyList{PyString => PyString});
/// # }
/// ```
///
/// And an other with i32:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// use rustypy::PyList;
/// let int_list = Box::new(PyList::from(vec![1i32; 5]));
/// let unpacked = unpack_pylist!(int_list; PyList{I32 => i32});
/// # }
/// ```
///
/// It can contain nested containers. A PyList which contains PyTuples which contain a list
/// of i64 PyTuples and a single f32:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// #    use rustypy::{PyList, PyArg};
/// #    let list = PyList::from(vec![
/// #        pytuple!(PyArg::PyList(Box::new(PyList::from(vec![
/// #                    pytuple!(PyArg::I64(1), PyArg::I64(2), PyArg::I64(3))]))),
/// #                 PyArg::F32(0.1)),
/// #        pytuple!(PyArg::PyList(Box::new(PyList::from(vec![
/// #                    pytuple!(PyArg::I64(3), PyArg::I64(2), PyArg::I64(1))]))),
/// #                 PyArg::F32(0.2))
/// #        ]).as_ptr();
/// // list from Python: [([(i64; 3)], f32)]
/// let list = unsafe { Box::new(PyList::from_ptr(list)) };
/// let unpacked = unpack_pylist!(list;
///     PyList{
///         PyTuple{(
///             {PyList{PyTuple{(I64, I64, I64,)}}}, F32,
///         )}
///     });
/// assert_eq!(vec![(vec![(1, 2, 3,)], 0.1), (vec![(3, 2, 1,)], 0.2)], unpacked);
/// # }
/// ```
///
#[macro_export]
macro_rules! unpack_pylist {
    ( $pylist:ident; PyList { $o:tt { $($t:tt)* } } ) => {{
        let mut unboxed = *($pylist);
        use std::collections::VecDeque;
        let mut list = VecDeque::with_capacity(unboxed.len());
        for _ in 0..unboxed.len() {
            match unboxed.pop() {
                Some(PyArg::$o(val)) => {
                    let inner = unpack_pylist!(val; $o { $($t)* });
                    list.push_front(inner);
                },
                Some(_) => _rustypy_abort_xtract_fail!("failed while converting pylist to vec"),
                None => {}
            }
        };
        Vec::from(list)
    }};
    ( $pytuple:ident; PyTuple { $t:tt } ) => {{
        let mut unboxed = *($pytuple);
        unpack_pytuple!(unboxed; $t)
    }};
    ( $pylist:ident; PyList{$t:tt => $type_:ty} ) => {{
        use rustypy::PyArg;
        let mut unboxed = *($pylist);
        use std::collections::VecDeque;
        let mut list = VecDeque::with_capacity(unboxed.len());
        for _ in 0..unboxed.len() {
            match unboxed.pop() {
                Some(PyArg::$t(val)) => { list.push_front(<$type_>::from(val)); },
                Some(_) => _rustypy_abort_xtract_fail!("failed while converting pylist to vec"),
                None => {}
            }
        };
        Vec::from(list)
    }};
    ( $pydict:ident; PyDict{$t} ) => {{
        unpack_pydict!( $pydict; PyDict{$t} )
    }};
    ( FROM_TUPLE: $pylist:ident; PyList{$t:tt => $type_:ty} ) => {{
        use rustypy::PyArg;
        let mut unboxed = &mut *($pylist);
        use std::collections::VecDeque;
        let mut list = VecDeque::with_capacity(unboxed.len());
        for _ in 0..unboxed.len() {
            match unboxed.pop() {
                Some(PyArg::$t(val)) => { list.push_front(<$type_>::from(val)); },
                Some(_) => _rustypy_abort_xtract_fail!("failed while converting pylist to vec"),
                None => {}
            }
        };
        Vec::from(list)
    }};
    ( FROM_TUPLE: $pylist:ident; PyList { $o:tt { $($t:tt)* } } ) => {{
        let mut unboxed = &mut *($pylist);
        use std::collections::VecDeque;
        let mut list = VecDeque::with_capacity(unboxed.len());
        for _ in 0..unboxed.len() {
            match unboxed.pop() {
                Some(PyArg::$o(val)) => {
                    let inner = unpack_pylist!(val; $o { $($t)* });
                    list.push_front(inner);
                },
                Some(_) => _rustypy_abort_xtract_fail!("failed while converting pylist to vec"),
                None => {}
            }
        };
        Vec::from(list)
    }};
}

#[no_mangle]
pub unsafe extern "C" fn pylist_new(len: usize) -> *mut PyList {
    let list = PyList { members: Vec::with_capacity(len) };
    list.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn pylist_push(list: &mut PyList, e: *mut PyArg) {
    list.push(*(Box::from_raw(e)));
}

#[no_mangle]
pub unsafe extern "C" fn pylist_len(list: &mut PyList) -> usize {
    list.len()
}

#[no_mangle]
pub extern "C" fn pylist_free(ptr: *mut PyList) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub unsafe extern "C" fn pylist_get_element(ptr: *mut PyList, index: usize) -> *mut PyArg {
    let list = &mut *ptr;
    Box::into_raw(Box::new(PyList::remove(list, index)))
}
