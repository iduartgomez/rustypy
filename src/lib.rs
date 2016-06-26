#![crate_type = "dylib"]

extern crate libc;
extern crate cpython;

use std::env;

use cpython::{
	Python,
};

pub fn setup_python(py: Python, path: Vec<&str>) {
	let sys = py.import("sys").unwrap();
	let version: String = sys.get(py, "version")
							 .unwrap().extract(py).unwrap();
	println!("loaded Python version {}", version);

	let mut dir = env::current_dir().unwrap();
	for p in path {
		dir.push(p)
	}
	let python_path = env::var_os("PYTHONPATH");
	let mut new_path;
	if let Some(ref path) = python_path {
		new_path = path.clone()
	} else {
		panic!("environmental variable PYTHONPATH not set")
	};
	new_path.push(":");
	new_path.push(dir.as_os_str());
	env::set_var("PYTHONPATH", &new_path);
}
