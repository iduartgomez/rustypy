#![allow(dead_code)]

extern crate libc;
extern crate cpython;
extern crate rustypy;

mod test_package;
use test_package::rustypy_pybind::PyModules;
use cpython::{Python};

#[test]
fn submodules() {
	let gil = Python::acquire_gil();
	let py = gil.python();
	let test_package: PyModules = PyModules::new(&py);
	test_package.root_module_1.root_module_1();
	test_package.root_module_2.root_module_2();
	test_package.firstdir.call_from_first.first_module();
	test_package.firstdir.subfirstdir.call_from_subfirst.subfirst_module();
}
