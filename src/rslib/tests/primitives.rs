extern crate libc;
extern crate cpython;
extern crate rustypy;

mod test_package;
use test_package::basics::rustypy_pybind::PyModules;
use cpython::Python;

#[test]
fn basics_primitives() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let module: PyModules = PyModules::new(&py);
    let basics = module.primitives;

    let arg = 1;
    let answ = basics.rust_bind_int_func(arg);
    assert_eq!(2, answ);

    let arg = 0.5;
    let answ = basics.rust_bind_float_func(arg);
    assert_eq!(1.0, answ);

    let arg = String::from("String from Rust, ");
    let answ = basics.rust_bind_str_func(arg);
    assert_eq!(String::from("String from Rust, added this in Python!"),
               answ);

    let arg = true;
    let answ = basics.rust_bind_bool_func(arg);
    assert_eq!(false, answ);

    let arg1 = String::from("String from Rust, ");
    let arg2 = 10;
    let answ = basics.rust_bind_tuple1((arg1, arg2));
    assert_eq!((String::from("String from Rust, added this in Python!"), 20),
               answ);

    let arg1 = String::from("String from Rust, ");
    let arg2 = true;
    let answ = basics.rust_bind_tuple2((arg1, arg2));
    assert_eq!((String::from("String from Rust, added this in Python!"), false),
               answ);

    let answ = basics.rust_bind_tuple3(0.5, true);
    assert_eq!((1.0, false), answ);
}

#[test]
fn pystring_operations() {
	use rustypy::PyString;

	let source = "test string";
	let owned_pystr = PyString::from(source).as_ptr();
	let back_from_py = unsafe { PyString::from_ptr_to_string(owned_pystr) };
	assert_eq!(back_from_py, "test string");
	{
		String::from(source);
	}
}
