# RustyPy
RustyPy is a code generator for generating binding functions between Rust and
Python files.

## Features
- Generate bindings in Rust targetting Python functions.
- Generate bindings in Python targetting Rust functions.

## Installation
To install RustyPy just use pip:
```
pip install rustypy
```
RustyPy requires Python 3.5 or more and works with Rust stable.

In Rust the package [cpython](https://github.com/dgrunwald/rust-cpython)
is required to initialize the package.

## Documentation
* [Python in Rust](https://github.com/iduartgomez/rustypy/wiki/Python-in-Rust)
* [Rust in Python](https://github.com/iduartgomez/rustypy/wiki/Rust-in-Python)
* [Type conversions](https://github.com/iduartgomez/rustypy/wiki/Type-conversions)

## Usage
RustyPy includes a command line interface to generate the code, which you can
embeed in your build chain if it's necessary.

### Generate Python bindings in Rust
You can execute the script writing:
```
$rustypy -h
```
or
```
$python -m rustypy -h
```
make sure that rustypy is in your current Python path. The help command has
all the information to generate bindings succesfully.

It also includes functions to generate bindings dynamically. In Python use:
```
from rustypy.pywrapper import bind_py_pckg_funcs

bind_py_pckg_funcs()
```
This function will generate the bindings for the package from which is
called from (so the package must be initiated placing an  \__init__.py file in
one of the parents folders).

More info: [Python in Rust](https://github.com/iduartgomez/rustypy/wiki/Python-in-Rust)

### Generate Rust bindings in Python
Due to the nature of Python this is done dynamically, so no files
are generated and the bindings are wrapped appropriately with their own callables
from Python.

```
from rustypy.rswrapper import bind_rs_crate_funcs

source_path = "/home/user/workspace/rs_test_lib"
compiled_lib = "/home/user/workspace/rs_test_lib/target/debug/libtest_lib.so"

lib_binds = bind_rs_crate_funcs(source_path, compiled_lib)

lib_binds.python_bind_my_function("Hello from Python!")
```

There is no concept of 'module' in C (which is the language used for interfacing)
so the functions cannot be namedspaced as you would in pure Rust. Read about
Rust FFI in [the book](https://doc.rust-lang.org/stable/book/ffi.html).

More info: [Rust in Python](https://github.com/iduartgomez/rustypy/wiki/Rust-in-Python)
Rust crate [documentation](https://iduartgomez.github.io/rustypy/).
