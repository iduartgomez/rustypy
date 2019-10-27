mod inner_types {
    /// Iterates over a a PyTuple and returns a corresponding Rust tuple.
    ///
    /// When destructured all inner data is moved out of the tuple which will retain the structure
    /// but will point to PyArg::None variant, so is safe to assume the PyTuple as consumed.
    ///
    /// Inner containers (ie. `PyList<PyArg(T)>`) are converted to the respective Rust analog
    /// (ie. `Vec<T>`) and require valid syntax for their respective unpack macro (ie.
    /// [unpack_pylist!](../rustypy/macro.unpack_pylist!.html)).
    ///
    /// # Examples
    ///
    /// Unpack a PyTuple which contains a two PyDict types with PyString keys
    /// and values of PyList<i64>:
    ///
    /// ```
    /// # #[macro_use] extern crate rustypy;
    /// # fn main(){
    /// # use rustypy::{PyDict, PyList, PyTuple, PyArg, PyString};
    /// # use std::collections::HashMap;
    /// # let mut hm = HashMap::new();
    /// # hm.insert(PyString::from("one"), vec![0_i32, 1, 2]);
    /// # hm.insert(PyString::from("two"), vec![3_i32, 2, 1]);
    /// # let mut pytuple = pytuple!(PyArg::PyDict(PyDict::from(hm.clone()).into_raw()),
    /// #                            PyArg::PyDict(PyDict::from(hm.clone()).into_raw())).into_raw();
    /// // tuple from Python: ({"one": [0, 1, 3]}, {"two": [3, 2, 1]})
    /// let unpacked = unpack_pytuple!(pytuple; ({PyDict{(PyString, PyList{I32 => i64})}},
    ///                                          {PyDict{(PyString, PyList{I32 => i64})}},));
    /// # }
    /// ```
    ///
    #[macro_export]
    macro_rules! unpack_pytuple {
        // FIXME: produces clippy warning on macro expansion due to usage `cnt`
        ($t:ident; ($($p:tt,)+) ) => {{
            use rustypy::{PyArg, PyTuple};
            use rustypy::pytypes::abort_and_exit;

            let mut cnt = 0;
            let mut pytuple = unsafe { PyTuple::from_ptr($t) };
            ($(
                unpack_pytuple!(pytuple; cnt; elem: $p)
            ,)*)
        }};
        ($t:ident; $i:ident; elem: ($($p:tt,)+))  => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::PyTuple(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    let mut val = unsafe { PyTuple::from_ptr(val) };
                    let mut cnt = 0;
                    ($(
                        unpack_pytuple!(val; cnt; elem: $p)
                    ,)*)
                },
                _ => abort_and_exit("failed while extracting a PyTuple inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: {PyDict{$u:tt}}) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::PyDict(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    unpack_pydict!(val; PyDict{$u})
                },
                _ => abort_and_exit("failed while extracting a PyDict inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: {PyList{$($u:tt)*}}) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::PyList(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    unpack_pylist!(val; PyList{$($u)*})
                },
                _ => abort_and_exit("failed while extracting a PyList inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: PyBool) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::PyBool(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val.to_bool()
                },
                _ => abort_and_exit("failed while extracting a PyBool inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: PyString) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::PyString(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val.to_string()
                },
                _ => abort_and_exit("failed while extracting a PyString inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: I64) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::I64(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a i64 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: I32) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::I32(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val.clone()
                },
                _ => abort_and_exit("failed while extracting a i32 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: I16) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::I16(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a i16 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: I8) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::I8(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a i8 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: U32) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::U32(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a u32 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: U16) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::U16(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a u16 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: U8) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::U8(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a u8 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: F32) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::F32(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a f32 inside a PyTuple"),
            }
        }};
        ($t:ident; $i:ident; elem: F64) => {{
            let e = $t.replace_elem($i).unwrap();
            match e {
                PyArg::F64(val) => {
                    $i += 1;
                    if $i == 0 {}; // stub to remove warning...
                    val
                },
                _ => abort_and_exit("failed while extracting a f64 inside a PyTuple"),
            }
        }};
    }

    /// Consumes a `PyList<PyArg(T)>` content as recived from Python (raw pointer)
    /// and returns a `Vec<T>` from it, no copies are performed in the process.
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
    /// let string_list = PyList::from(vec!["Python", "in", "Rust"]).into_raw();
    /// let unpacked = unpack_pylist!(string_list; PyList{PyString => PyString});
    /// # }
    /// ```
    ///
    /// And an other with i32:
    ///
    /// ```
    /// # #[macro_use] extern crate rustypy;
    /// # fn main(){
    /// # use rustypy::PyList;
    /// # let int_list = PyList::from(vec![1i32; 5]).into_raw();
    /// // list from python: [1, 1, 1, 1, 1]
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
    /// #        pytuple!(PyArg::PyList(PyList::from(vec![
    /// #                    pytuple!(PyArg::I64(1), PyArg::I64(2), PyArg::I64(3))]).into_raw()),
    /// #                 PyArg::F32(0.1)),
    /// #        pytuple!(PyArg::PyList(PyList::from(vec![
    /// #                    pytuple!(PyArg::I64(3), PyArg::I64(2), PyArg::I64(1))]).into_raw()),
    /// #                 PyArg::F32(0.2))
    /// #        ]).into_raw();
    /// // list from Python: [([(i64; 3)], f32)]
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
            use rustypy::{PyList, PyArg};
            use rustypy::pytypes::abort_and_exit;
            use std::collections::VecDeque;

            let mut unboxed = unsafe { PyList::from_ptr($pylist) };
            let mut list = VecDeque::with_capacity(unboxed.len());
            for _ in 0..unboxed.len() {
                match unboxed.pop() {
                    Some(PyArg::$o(val)) => {
                        let inner = unpack_pylist!(val; $o { $($t)* });
                        list.push_front(inner);
                    },
                    Some(_) => abort_and_exit("failed while converting pylist to vec"),
                    None => {}
                }
            };
            Vec::from(list)
        }};
        ( $pytuple:ident; PyTuple { $t:tt } ) => {{
            unpack_pytuple!($pytuple; $t)
        }};
        ( $pylist:ident; PyList{$t:tt => $type_:ty} ) => {{
            use rustypy::{PyList, PyArg};
            use rustypy::pytypes::abort_and_exit;
            use std::collections::VecDeque;

            let mut unboxed = unsafe { PyList::from_ptr($pylist) };
            let mut list = VecDeque::with_capacity(unboxed.len());
            for _ in 0..unboxed.len() {
                match unboxed.pop() {
                    Some(PyArg::$t(val)) => { list.push_front(<$type_>::from(val)); },
                    Some(_) => abort_and_exit("failed while converting pylist to vec"),
                    None => {}
                }
            };
            Vec::from(list)
        }};
        ( $pydict:ident; PyDict{$t:tt} ) => {{
            unpack_pydict!( $pydict; PyDict{$t} )
        }};
    }

    /// Consumes the content of a `PyDict<K, T>` as received from Python (raw pointer)
    /// and returns a `HashMap<K, T>` from it, no copies are performed in the process.
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
    /// # use std::iter::FromIterator;
    /// use rustypy::{PyDict, PyString};
    /// use std::collections::HashMap;
    ///
    /// let mut hm = HashMap::from_iter(vec![(0_i32, "Hello"), (1_i32, " "), (2_i32, "World!")]);
    /// let dict = PyDict::from(hm).into_raw();
    /// let unpacked = unpack_pydict!(dict; PyDict{(i32, PyString => String)});
    /// # }
    /// ```
    ///
    /// With nested container types:
    ///
    /// ```
    /// # #[macro_use] extern crate rustypy;
    /// # #[allow(unused_variables)]
    /// # fn main(){
    /// # use rustypy::{PyDict, PyString, PyArg};
    /// # use std::collections::HashMap;
    /// # let mut hm0 = HashMap::new();
    /// # for &(k, v) in &[(0_i32, "Hello"), (1, " "), (2, "World!")] {
    /// #     hm0.insert(k, v);
    /// # }
    /// # let mut hm1 = HashMap::new();
    /// # for i in 0..3_i64 {
    /// #     let k = format!("#{}", i);
    /// #     let v = PyArg::from(hm0.clone());
    /// #     let l = PyArg::from(vec![0_i64; 3]);
    /// #     hm1.insert(PyString::from(k), pytuple!(l, v));
    /// # }
    /// # let dict = PyDict::from(hm1).into_raw();
    /// // dict from Python: {str: ([u64], {i32: str})} as *mut size_t
    /// let unpacked = unpack_pydict!(dict;
    ///     PyDict{(PyString, PyTuple{({PyList{I64 => i64}},
    ///                                {PyDict{(i32, PyString => String)}},)})}
    ///     );
    /// # }
    /// ```
    ///
    #[macro_export]
    macro_rules! unpack_pydict {
        ( $pydict:ident; PyDict{($kt:ty, $o:tt { $($t:tt)* })} ) => {{
            use rustypy::{PyArg, PyDict};
            use rustypy::pytypes::abort_and_exit;
            use std::collections::HashMap;

            let mut unboxed = unsafe { *(Box::from_raw($pydict as *mut PyDict<$kt>)) };
            let mut dict = HashMap::new();
            for (k, v) in unboxed.drain() {
                match v {
                    PyArg::$o(val) => {
                        let inner = unpack_pydict!(val; $o { $($t)* });
                        dict.insert(k, inner);
                    }
                    _ => abort_and_exit("failed while converting PyDict to HashMap")
                }
            }
            dict
        }};
        ( $pytuple:ident; PyTuple { $t:tt } ) => {{
            unpack_pytuple!($pytuple; $t)
        }};
        ( $pylist:ident; PyList{ $($u:tt)* } ) => {{
            unpack_pylist!( $pylist; PyList{ $($u)* } )
        }};
        ( $pydict:ident; PyDict{($kt:ty, $t:tt => $type_:ty)} ) => {{
            use rustypy::{PyArg, PyDict};
            use rustypy::pytypes::abort_and_exit;
            use std::collections::HashMap;

            let mut unboxed = unsafe { *(Box::from_raw($pydict as *mut PyDict<$kt>)) };
            let mut dict = HashMap::new();
            for (k, v) in unboxed.drain() {
                match v {
                    PyArg::$t(val) => { dict.insert(k, <$type_>::from(val)); },
                    _ => abort_and_exit("failed while converting PyDict to HashMap"),
                }
            }
            dict
        }};
    }
}

/// Converts python data to `std` Rust collections or tuples, moving the data:
/// * Consumes the content of a `PyDict<K, T>` as received from Python (raw pointer)
///   and returns a `HashMap<K, T>` from it.
/// * Consumes a `PyList<PyArg(T)>` content as recived from Python (raw pointer)
///   and returns a `Vec<T>` from it.
/// * Iterates over a a PyTuple and returns a corresponding Rust tuple. Moves the data
///   leaving an empty PyTuple.
///
/// Inner containers (ie. `PyList<PyArg(T)>`) are converted to the respective Rust analog
/// (ie. `Vec<T>`)
///
/// # Examples
///
/// Unpack a PyTuple from Python:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use rustypy::{PyDict, PyList, PyTuple, PyArg, PyString};
/// # use std::collections::HashMap;
/// # let mut hm = HashMap::new();
/// # hm.insert(PyString::from("one"), vec![0_i32, 1, 2]);
/// # hm.insert(PyString::from("two"), vec![3_i32, 2, 1]);
/// # let mut pytuple = pytuple!(PyArg::PyDict(PyDict::from(hm.clone()).into_raw()),
/// #                            PyArg::PyDict(PyDict::from(hm.clone()).into_raw())).into_raw();
/// // tuple from Python: ({"one": [0, 1, 3]}, {"two": [3, 2, 1]})
/// let unpacked = unpack_pytuple!(pytuple; ({PyDict{(PyString, PyList{I32 => i64})}},
///                                          {PyDict{(PyString, PyList{I32 => i64})}},));
/// # }
/// ```
///
/// Unpack a PyList from Python:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use rustypy::PyList;
/// # let int_list = PyList::from(vec![1i32; 5]).into_raw();
/// // list from python: [1, 1, 1, 1, 1]
/// let unpacked = unpack_pytype!(int_list; PyList{I32 => i32});
/// # }
/// ```
///
/// Unpack a PyDict from Python:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # fn main(){
/// # use std::iter::FromIterator;
/// # use rustypy::{PyDict, PyString};
/// # use std::collections::HashMap;
/// #
/// # let mut hm = HashMap::from_iter(vec![(0_i32, "Hello"), (1_i32, " "), (2_i32, "World!")]);
/// # let dict = PyDict::from(hm).into_raw();
/// let unpacked = unpack_pytype!(dict; PyDict{(i32, PyString => String)});
/// # }
/// ```
///
/// You can combine different containers and nest them:
///
/// ```
/// # #[macro_use] extern crate rustypy;
/// # #[allow(unused_variables)]
/// # fn main(){
/// # use rustypy::{PyDict, PyString, PyArg};
/// # use std::collections::HashMap;
/// # let mut hm0 = HashMap::new();
/// # for &(k, v) in &[(0_i32, "Hello"), (1, " "), (2, "World!")] {
/// #     hm0.insert(k, v);
/// # }
/// # let mut hm1 = HashMap::new();
/// # for i in 0..3_i64 {
/// #     let k = format!("#{}", i);
/// #     let v = PyArg::from(hm0.clone());
/// #     let l = PyArg::from(vec![0_i64; 3]);
/// #     hm1.insert(PyString::from(k), pytuple!(l, v));
/// # }
/// # let dict = PyDict::from(hm1).into_raw();
/// // dict from Python: {str: ([u64], {i32: str})} as *mut size_t
/// let unpacked = unpack_pytype!(dict;
///     PyDict{(PyString, PyTuple{({PyList{I64 => i64}},
///                                {PyDict{(i32, PyString => String)}},)})}
///     );
/// # }
/// ```
#[macro_export]
macro_rules! unpack_pytype {
    ($t:ident; ($($p:tt,)+)) => {
        unpack_pytuple!($t; ($($p,)*))
    };
    ($pylist:ident; PyList{$t:tt => $type_:ty}) => {
        unpack_pylist!($pylist; PyList{$t => $type_})
    };
    ($pylist:ident; PyList { $o:tt { $($t:tt)* } }) => {
        unpack_pylist!($pylist; PyList { $o { $($t)* } })
    };
    ($pydict:ident; PyDict{($kt:ty, $o:tt { $($t:tt)* })}) => {
        unpack_pydict!($pydict; PyDict{($kt, $o { $($t)* })})
    };
    ($pydict:ident; PyDict{($kt:ty, $t:tt => $type_:ty)}) => {
        unpack_pydict!($pydict; PyDict{($kt, $t => $type_)})
    };
}

#[cfg(test)]
mod tests {
    use crate as rustypy;
    use rustypy::*;

    #[test]
    fn pytuple_macro() {
        let pytuple = pytuple!(
            PyArg::PyBool(PyBool::from(false)),
            PyArg::PyString(PyString::from("test")),
            PyArg::I64(55i64)
        )
        .into_raw();
        let unpacked = unpack_pytype!(pytuple; (PyBool, PyString, I64,));
        assert_eq!((false, String::from("test"), 55i64), unpacked);
    }

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
}
