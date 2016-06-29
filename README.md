# RustyPy
RustyPy is a code generator for generating binding functions between Rust and
Python files.

## Features
- Generate bindings in Rust targetting Python functions.
- (TODO) ~~Generate bindings in Python targetting Rust functions.~~ 

## Installation
To install RustyPy just use pip:
```
pip install rustypy
```
RustyPy requires Python 3.5 or more and works with Rust stable.

In Rust the package **cpython** is required.

## Documentation
* [Python to Rust](https://github.com/iduartgomez/rustypy/wiki/Python-in-Rust)
* [Type conversions](https://github.com/iduartgomez/rustypy/wiki/Type-conversions)

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
