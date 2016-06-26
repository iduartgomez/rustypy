# RustyPy
RustyPy is a code generator for generating binding functions between Rust and
Python files.

## Features
- Generate bindings in Rust targetting Python functions.
- ~~Generate bindings in Python targetting Rust functions.~~ (TODO)

## Installation
To install RustyPy just use pip:
```
pip install rustypy
```
RustyPy requires Python 3.5 or more and works with Rust stable.

In Rust the package **cpython** is required.

## Usage
RustyPy includes a command line interface to generate the code, which you can
embeed in your build chain if it's necessary.

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

### Python to Rust examples
RustyPy requires Python 3.5 at least, and the functions that are to be  
Assume the following package structure:
```
/example_package/
 |
 |__/test/
 |   |...
 |__/src/
     /subdir/
     ||__  sub.py
     |__  __init__.py
     |__  funcs.py
     |__  lib.rs
```

The content of funcs.py is:
```
from rustypy include rust_bind

@rust_bind
def add_one(arg: int) -> int:
  result = arg + 1
  return result

def rust_bind_hello() -> None:
  print('Hello from Rust!')
```

The content of sub.py is:
```
from rustypy include rust_bind

@rust_bind
def print_something(arg: str) -> None:
  print(str)
```

Any function which has the **rust\_bind_** prefix or decorated by the
**rust_bind** decorator will be parsed and a new file (rustypy\_pybind.rs)
containing the bindings will be generated at the package root (in this case in **/example\_package/src/rustypy\_pybind.rs**).

Then we call rustypy:
```
$ rustypy -p python /example_package/src
```

or

```
$ cd example_package
example_package$ pip install -e .
example_package$ rustypy -p python example_package
```

The second form installs our package in development mode, in this case we can
just provide the name of the package instead of the absolute path to it.

To use our package from Rust we just have to import the _package manager_ in
the wanted module (lib.rs for example):
```
extern crate cpython;

mod rustypy_pybind;

use rustypy_pybind::PyModules;
use cpython::Python;

// setup Python interpreter
let gil = Python::acquire_gil();
let py = gil.python();

// setup the module manager
let example_package = PyModules::new(py);

// call functions
example_package.funcs.rust_bind_hello(py);
let num = 10;
let result = example_package.funcs.add_one(py, num);
assert_eq!(11, result);
let arg = String::from("Hello from Rust!");
example_package.subdir.sub.print_something(py, arg);
```

The module manager uses object notation to structure the subpackages and
functions. For more examples you can check the tests directory.

## Types conversions between Python and Rust
You can use the **typing** Python module to compose complex types from primitive
types. This are the primitive types conversions:

Python      |     Rust
------------|-------------
int         |  c_long
float       |  c_double
string      |  String
bool        |  bool

Container types supported:

Python      |     Rust
------------|-------------
list        |  Vec
dict        |  HashMap
tuple       |  tuple

Any other type will be extracted as a PyObject and left for the client to
manage. You can leverage this to use generics.

And a couple examples of complex types in Python:
```
import typing

@rust_bind
def dict(dict_arg: typing.Dict[str, int]) -> typing.Dict[str, int]:
    """
    Use in Rust:
      dict.insert(String::from("first_key"), 1);
      let answ = python_modules.dict(py, dict);"""

    for k, v in dict_arg.items():
        dict_arg[k] = v + 1
    return dict_arg

T = typing.TypeVar('A', int, str)

@rust_bind
def generic(g_arg: T) -> T:
    """
    Use in Rust:
      let arg: PyLong = 0.to_py_object(py);
    	let answ = basics.generic1(py, arg.into_object());
    	assert_eq!(0, answ.extract::<c_long>(py).unwrap());"""

    assert isinstance(g_arg, int) or isinstance(g_arg, str), \
        'provided argument is not an int or str'
    return g_arg
```

Remember that the official CPython implementaion does not have static type
checking by default, but the type annotations are used to infeer the equivalent
types returned in Rust.
