#![allow(
	unused_imports,
	dead_code
)]

extern crate libc;
extern crate cpython;
extern crate rustypy;

use cpython::{
	Python,
};

mod test_package;
use test_package::basics::PyModules;

use rustypy::setup_python;

#[test]
fn basics_primitives(){
	let gil = Python::acquire_gil();
	let py = gil.python();
	setup_python(py, vec!["tests", "test_package", "basics"]);
	let module: PyModules = PyModules::new(py);
	let basics = module.primitives;

	let arg = 1;
	let answ = basics.rust_bind_int_func(py, arg);
	assert_eq!(2, answ);

	let arg = 0.5;
	let answ = basics.rust_bind_float_func(py, arg);
	assert_eq!(1.0, answ);

	let arg = String::from("String from Rust, ");
	let answ = basics.rust_bind_str_func(py, arg);
	assert_eq!(
		String::from("String from Rust, added this in Python!"),
		answ);

	let arg = true;
	let answ = basics.rust_bind_bool_func(py, arg);
	assert_eq!(false, answ);

	let arg1 = String::from("String from Rust, ");
	let arg2 = 10;
	let answ = basics.rust_bind_tuple1(py, (arg1, arg2));
	assert_eq!(
		(String::from("String from Rust, added this in Python!"), 20),
		answ);

	let arg1 = String::from("String from Rust, ");
	let arg2 = true;
	let answ = basics.rust_bind_tuple2(py, (arg1, arg2));
	assert_eq!(
		(String::from("String from Rust, added this in Python!"), false),
		answ);

	let answ = basics.rust_bind_tuple3(py, 0.5, true);
	assert_eq!((1.0, false), answ);
}
