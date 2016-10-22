# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python."""

import os
import sys
import pkg_resources
import re
import types
import typing
from string import Template
from collections import deque, namedtuple

##### CFFI #####
import ctypes
from ctypes import POINTER, ARRAY, c_void_p

global c_backend
c_backend = None

RS_TYPE_CONVERSION = {
    'c_float': 'float',
    'c_double': 'double',
    'c_short': 'int',
    'c_int': 'int',
    'c_long': 'int',
    'c_longlong': 'int',
    'c_ushort': 'int',
    'c_uint': 'int',
    'c_ulong': 'int',
    'u32': 'int',
    'u16': 'int',
    'u8': 'int',
    'i64': 'int',
    'i32': 'int',
    'i16': 'int',
    'i8': 'int',
    'f32': 'float',
    'f64': 'double',
    'PyTuple': 'tuple',
    'PyBool': 'bool',
    'PyString': 'str',
    'PyList': 'list',
    'usize': 'POINTER',
    'size_t': 'POINTER',
    'void': 'None',
}


class PyString_RS(ctypes.Structure):
    pass


class PyBool_RS(ctypes.Structure):
    pass


class PyTuple_RS(ctypes.Structure):
    pass


class PyList_RS(ctypes.Structure):
    pass


# Dictionary interfaces:
class PyDict_RS(ctypes.Structure):
    pass


class KeyType_RS(ctypes.Structure):
    pass


class DrainPyDict_RS(ctypes.Structure):
    pass
# END dictionary interfaces


class Raw_RS(ctypes.Structure):
    pass


class PyArg_RS(ctypes.Structure):
    pass


class KrateData_RS(ctypes.Structure):
    pass


def config_ctypes():
    # Crate parsing functions
    c_backend.krate_data_new.restype = POINTER(KrateData_RS)
    c_backend.krate_data_free.argtypes = (POINTER(KrateData_RS), )
    c_backend.krate_data_free.restype = c_void_p
    c_backend.krate_data_len.argtypes = (POINTER(KrateData_RS), )
    c_backend.krate_data_len.restype = ctypes.c_size_t
    c_backend.krate_data_iter.argtypes = (
        POINTER(KrateData_RS), ctypes.c_size_t)
    c_backend.krate_data_iter.restype = POINTER(PyString_RS)
    c_backend.parse_src.argtypes = (
        POINTER(PyString_RS), POINTER(KrateData_RS))
    c_backend.parse_src.restype = ctypes.c_uint

    # String related functions
    c_backend.pystring_new.argtypes = (ctypes.c_char_p, )
    c_backend.pystring_new.restype = POINTER(PyString_RS)
    c_backend.pystring_free.argtypes = (POINTER(PyString_RS), )
    c_backend.pystring_free.restype = c_void_p
    c_backend.pystring_get_str.argtypes = (POINTER(PyString_RS), )
    c_backend.pystring_get_str.restype = ctypes.c_char_p

    # Bool related functions
    c_backend.pybool_new.argtypes = (ctypes.c_byte, )
    c_backend.pybool_new.restype = POINTER(PyBool_RS)
    c_backend.pybool_free.argtypes = (POINTER(PyBool_RS), )
    c_backend.pybool_free.restype = c_void_p
    c_backend.pybool_get_val.argtypes = (POINTER(PyBool_RS), )
    c_backend.pybool_get_val.restype = ctypes.c_byte

    # Tuple related functions
    c_backend.pytuple_new.argtypes = (ctypes.c_size_t, )
    c_backend.pytuple_new.restype = POINTER(PyTuple_RS)
    c_backend.pytuple_push.argtypes = (
        POINTER(PyTuple_RS), POINTER(PyTuple_RS))
    c_backend.pytuple_push.restype = c_void_p
    c_backend.pytuple_len.argtypes = (POINTER(PyTuple_RS),)
    c_backend.pytuple_len.restype = ctypes.c_size_t
    c_backend.pytuple_free.argtypes = (POINTER(PyTuple_RS), )
    c_backend.pytuple_free.restype = c_void_p
    c_backend.pytuple_get_element.argtypes = (
        POINTER(PyTuple_RS), ctypes.c_size_t)
    c_backend.pytuple_get_element.restype = POINTER(PyArg_RS)

    # List related functions
    c_backend.pylist_new.argtypes = (ctypes.c_size_t, )
    c_backend.pylist_new.restype = POINTER(PyList_RS)
    c_backend.pylist_push.argtypes = (POINTER(PyList_RS), POINTER(PyArg_RS))
    c_backend.pylist_push.restype = c_void_p
    c_backend.pylist_len.argtypes = (POINTER(PyList_RS), )
    c_backend.pylist_len.restype = ctypes.c_size_t
    c_backend.pylist_free.argtypes = (POINTER(PyList_RS), )
    c_backend.pylist_free.restype = c_void_p
    c_backend.pylist_get_element.argtypes = (
        POINTER(PyList_RS), ctypes.c_size_t)
    c_backend.pylist_get_element.restype = POINTER(PyArg_RS)

    # Dict related functions
    c_backend.pydict_new.argtypes = (POINTER(KeyType_RS), )
    c_backend.pydict_new.restype = POINTER(PyDict_RS)
    c_backend.pydict_free.argtypes = (POINTER(PyDict_RS), POINTER(KeyType_RS))
    c_backend.pydict_free.restype = c_void_p
    c_backend.pydict_get_key_type.argtypes = (ctypes.c_uint, )
    c_backend.pydict_get_key_type.restype = POINTER(KeyType_RS)
    c_backend.pydict_insert.argtypes = (POINTER(PyDict_RS), POINTER(
        KeyType_RS), POINTER(PyArg_RS), POINTER(PyArg_RS))
    c_backend.pydict_insert.restype = c_void_p
    c_backend.pydict_get_element.restype = POINTER(PyArg_RS)
    c_backend.pydict_get_drain.argtypes = (
        POINTER(PyDict_RS), POINTER(KeyType_RS))
    c_backend.pydict_get_drain.restype = POINTER(DrainPyDict_RS)
    c_backend.pydict_drain_element.argtypes = (
        POINTER(DrainPyDict_RS), POINTER(KeyType_RS))
    c_backend.pydict_drain_element.restype = POINTER(PyTuple_RS)

    # Wrap type in PyArg enum
    c_backend.pyarg_from_str.argtypes = (ctypes.c_char_p,)
    c_backend.pyarg_from_str.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_int.argtypes = (ctypes.c_longlong,)
    c_backend.pyarg_from_int.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_ulonglong.argtypes = (ctypes.c_ulonglong,)
    c_backend.pyarg_from_ulonglong.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_float.argtypes = (ctypes.c_float,)
    c_backend.pyarg_from_float.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_double.argtypes = (ctypes.c_double,)
    c_backend.pyarg_from_double.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_bool.argtypes = (ctypes.c_byte,)
    c_backend.pyarg_from_bool.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_pytuple.argtypes = (POINTER(PyTuple_RS),)
    c_backend.pyarg_from_pytuple.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_pylist.argtypes = (POINTER(PyList_RS),)
    c_backend.pyarg_from_pylist.restype = POINTER(PyArg_RS)
    c_backend.pyarg_from_pydict.argtypes = (POINTER(PyDict_RS),)
    c_backend.pyarg_from_pydict.restype = POINTER(PyArg_RS)
    # Get val from enum
    c_backend.pyarg_extract_owned_int.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_int.restype = ctypes.c_longlong
    c_backend.pyarg_extract_owned_ulonglong.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_ulonglong.restype = ctypes.c_ulonglong
    c_backend.pyarg_extract_owned_float.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_float.restype = ctypes.c_float
    c_backend.pyarg_extract_owned_double.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_double.restype = ctypes.c_double
    c_backend.pyarg_extract_owned_bool.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_bool.restype = POINTER(PyBool_RS)
    c_backend.pyarg_extract_owned_str.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_str.restype = POINTER(PyString_RS)
    c_backend.pyarg_extract_owned_tuple.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_tuple.restype = POINTER(PyTuple_RS)
    c_backend.pyarg_extract_owned_list.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_list.restype = POINTER(PyList_RS)
    c_backend.pyarg_extract_owned_dict.argtypes = (POINTER(PyArg_RS),)
    c_backend.pyarg_extract_owned_dict.restype = POINTER(PyDict_RS)


def load_rust_lib(recmpl=False):
    ext = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
    pre = {'win32': ''}.get(sys.platform, 'lib')
    lib = pkg_resources.resource_filename(
        'rslib', "{}rustypy{}".format(pre, ext))
    if (not os.path.exists(lib)) or recmpl:
        print("   library not found at: {}".format(lib))
        print("   compiling with Cargo")
        import subprocess
        path = os.path.dirname(lib)
        subprocess.run(['cargo', 'build', '--release'], cwd=path)
        import shutil
        cp = os.path.join(path, 'target', 'release',
                          "librustypy{}".format(ext))
        if os.path.exists(lib):
            os.remove(lib)
        shutil.copy(cp, path)
        load_rust_lib()
    else:
        from .__init__ import __version__ as curr_ver
        # check that is the same version
        lib_ver = curr_ver
        # load the library
        if lib_ver != curr_ver:
            compile_rust_lib(recmpl=True)
        else:
            globals()['c_backend'] = ctypes.cdll.LoadLibrary(lib)
            config_ctypes()

# ==================== #
#   Type Wrappers      #
# ==================== #


Float = type('Float', (float,), {'_definition': ctypes.c_float})
Double = type('Double', (float,), {'_definition': ctypes.c_double})
UnsignedLongLong = type('UnsignedLongLong', (int,), {
                        '_definition': ctypes.c_ulonglong})


class MissingTypeHint(TypeError):
    pass


class PythonObject(object):

    def __init__(self, ptr):
        self._ptr = ptr

    def get_rs_obj(self):
        return self._ptr


def _dangling_pointer(*args, **kwargs):
    raise ReferenceError(
        "rustypy: the underlying Rust type has been dropped")


class PyString(PythonObject):

    def free(self):
        c_backend.pystring_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_dict', _dangling_pointer)

    def to_str(self):
        """Consumes the wrapper and returns a raw c_char pointer.
        Afterwards is not necessary to destruct it as it has already
        been consumed."""
        val = c_backend.pystring_get_str(self._ptr)
        return val.decode("utf-8")

    @staticmethod
    def from_str(s: str):
        return c_backend.pystring_new(s.encode("utf-8"))


class PyBool(PythonObject):

    def free(self):
        c_backend.pybool_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_dict', _dangling_pointer)

    def to_bool(self):
        val = c_backend.pybool_get_val(self._ptr)
        if val == 0:
            val = False
        else:
            val = True
        return val

    @staticmethod
    def from_bool(val: bool):
        if val is True:
            return c_backend.pybool_new(1)
        else:
            return c_backend.pybool_new(0)


def _to_pybool(arg):
    if arg:
        return c_backend.pyarg_from_bool(1)
    else:
        return c_backend.pyarg_from_bool(0)


def _to_pystring(arg):
    return c_backend.pyarg_from_str(arg.encode("utf-8"))


def _to_pytuple(sig):
    def dec(arg):
        return c_backend.pyarg_from_pytuple(PyTuple.from_tuple(arg, sig))
    return dec


def _to_pylist(sig):
    def dec(arg):
        return c_backend.pyarg_from_pylist(PyList.from_list(arg, sig))
    return dec


def _to_pydict(sig):
    def dec(arg):
        d = PyDict.from_dict(arg, sig)
        return c_backend.pyarg_from_pydict(d)
    return dec


class PyTuple(PythonObject):

    def __init__(self, ptr, signature, call_fn=None):
        self._ptr = ptr
        if not signature:
            raise MissingTypeHint(
                "rustypy: missing type hint for PyTuple unpacking in Python")
        self.sig = signature
        self.call_fn = call_fn

    def free(self):
        c_backend.pytuple_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_dict', _dangling_pointer)

    def to_tuple(self, depth=0):
        arity = c_backend.pytuple_len(self._ptr)
        if arity != len(self.sig.__tuple_params__) and self.call_fn:
            raise TypeError("rustypy: the type hint for returning tuple of fn `{}` "
                            "and the return tuple value are not of "
                            "the same length".format(self.call_fn._fn_name))
        elif arity != len(self.sig.__tuple_params__):
            raise TypeError(
                "rustypy: type hint for PyTuple is of wrong length")
        tuple_elems = []
        for last, arg_t in enumerate(self.sig.__tuple_params__):
            pyarg = c_backend.pytuple_get_element(self._ptr, last)
            if arg_t is str:
                content = c_backend.pyarg_extract_owned_str(pyarg)
                pytype = c_backend.pystring_get_str(content).decode("utf-8")
            elif arg_t is bool:
                b = PyBool(c_backend.pyarg_extract_owned_bool(pyarg))
                pytype = b.to_bool()
                b.free()
            elif arg_t is int:
                pytype = c_backend.pyarg_extract_owned_int(pyarg)
            elif arg_t is UnsignedLongLong:
                pytype = c_backend.pyarg_extract_owned_ulonglong(pyarg)
            elif arg_t is Double or arg_t is float:
                pytype = c_backend.pyarg_extract_owned_double(pyarg)
            elif arg_t is Float:
                pytype = c_backend.pyarg_extract_owned_float(pyarg)
            elif issubclass(arg_t, typing.Tuple):
                ptr = c_backend.pyarg_extract_owned_tuple(pyarg)
                t = PyTuple(ptr, arg_t)
                pytype = t.to_tuple(depth=depth + 1)
                t.free()
            elif issubclass(arg_t, typing.List):
                ptr = c_backend.pyarg_extract_owned_list(pyarg)
                l = PyList(ptr, arg_t)
                pytype = l.to_list(depth=depth + 1)
                l.free()
            elif issubclass(arg_t, typing.Dict):
                ptr = c_backend.pyarg_extract_owned_dict(pyarg)
                d = PyDict(ptr, arg_t)
                pytype = d.to_dict(depth=depth + 1)
                d.free()
            else:
                raise TypeError("rustypy: subtype `{t}` of Tuple type is \
                                not supported".format(t=arg_t))
            tuple_elems.append(pytype)
        return tuple(tuple_elems)

    @staticmethod
    def from_tuple(source: tuple, sig):
        next_e = None
        cnt = len(source) - 1
        for i in range(0, len(source)):
            cnt = cnt - i
            arg_t = sig.__tuple_params__[cnt]
            last = source[cnt]
            if arg_t is str:
                pyarg = _to_pystring(last)
            elif arg_t is bool:
                pyarg = _to_pystring(last)
            elif arg_t is int:
                pyarg = c_backend.pyarg_from_int(last)
            elif arg_t is Double or arg_t is float:
                pyarg = c_backend.pyarg_from_double(last)
            elif arg_t is Float:
                pyarg = c_backend.pyarg_from_float(last)
            elif issubclass(arg_t, typing.Tuple):
                pyarg = _to_pytuple(arg_t)(last)
            elif issubclass(arg_t, typing.List):
                pyarg = _to_pylist(arg_t)(last)
            elif issubclass(arg_t, typing.Dict):
                pyarg = _to_pydict(arg_t)(last)
            else:
                raise TypeError("rustypy: subtype `{t}` of Tuple type is \
                                not supported".format(t=arg_t))
            prev_e = c_backend.pytuple_new(cnt, pyarg)
            if next_e:
                c_backend.pytuple_push(next_e, prev_e)
            next_e = prev_e
        return prev_e


class PyList(PythonObject):

    def __init__(self, ptr, signature, call_fn=None):
        self._ptr = ptr
        self._len = c_backend.pylist_len(self._ptr)
        if not signature:
            raise MissingTypeHint(
                "rustypy: missing type hint for PyList unpacking in Python")
        self.sig = signature
        self.call_fn = call_fn

    def free(self):
        c_backend.pylist_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_dict', _dangling_pointer)

    def to_list(self, depth=0):
        arg_t = self.sig.__args__[0]
        pylist = deque()
        last = self._len - 1
        if arg_t is str:
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                content = c_backend.pyarg_extract_owned_str(pyarg)
                pylist.appendleft(
                    c_backend.pystring_get_str(content).decode("utf-8"))
                last -= 1
        elif arg_t is bool:
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                b = PyBool(c_backend.pyarg_extract_owned_bool(pyarg))
                pylist.appendleft(b.to_bool())
                b.free()
                last -= 1
        elif arg_t is int:
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                content = c_backend.pyarg_extract_owned_int(pyarg)
                pylist.appendleft(content)
                last -= 1
        elif arg_t is Double or arg_t is float:
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                content = c_backend.pyarg_extract_owned_double(pyarg)
                pylist.appendleft(content)
                last -= 1
        elif arg_t is Float:
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                content = c_backend.pyarg_extract_owned_float(pyarg)
                pylist.appendleft(content)
                last -= 1
        elif issubclass(arg_t, typing.Tuple):
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                ptr = c_backend.pyarg_extract_owned_tuple(pyarg)
                t = PyTuple(ptr, arg_t)
                pylist.appendleft(t.to_tuple(depth=depth + 1))
                t.free()
                last -= 1
        elif issubclass(arg_t, typing.List):
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                ptr = c_backend.pyarg_extract_owned_list(pyarg)
                l = PyList(ptr, arg_t)
                pylist.appendleft(l.to_list(depth=depth + 1))
                l.free()
                last -= 1
        elif issubclass(arg_t, typing.Dict):
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                ptr = c_backend.pyarg_extract_owned_dict(pyarg)
                d = PyDict(ptr, arg_t)
                pylist.appendleft(d.to_dict(depth=depth + 1))
                d.free()
                last -= 1
        else:
            raise TypeError("rustypy: subtype `{t}` of List type is \
                            not supported".format(t=arg_t))
        return list(pylist)

    @staticmethod
    def from_list(source: list, sig):
        arg_t = sig.__args__[0]
        if arg_t is str:
            fn = _to_pystring
        elif arg_t is bool:
            fn = _to_pybool
        elif arg_t is int:
            fn = c_backend.pyarg_from_int
        elif arg_t is Double or arg_t is float:
            fn = c_backend.pyarg_from_double
        elif arg_t is Float:
            fn = c_backend.pyarg_from_float
        elif issubclass(arg_t, typing.Tuple):
            fn = _to_pytuple(arg_t)
        elif issubclass(arg_t, typing.List):
            fn = _to_pylist(arg_t)
        elif issubclass(arg_t, typing.Dict):
            fn = _to_pydict(arg_t)
        else:
            raise TypeError("rustypy: subtype {t} of List type is \
                            not supported".format(t=arg_t))

        pylist = c_backend.pylist_new(len(source))
        for e in source:
            c_backend.pylist_push(pylist, fn(e))
        return pylist

import abc


class HashableTypeABC(abc.ABCMeta):
    __allowed = ['i64', 'i32', 'i16', 'i8',
                 'u64', 'u32', 'u16', 'u8',
                 'PyString', 'PyBool']

    _doc = """Represents a hashable supported Rust type.
Args:
    t (str): String representing the type, the following are supported:
        i64, i32, i16, i8, u32, u16, u8, PyString, PyBool
"""

    def __call__(cls, t):
        if t not in cls.__allowed:
            raise TypeError("rustypy: dictionary key must be one of the \
            following types: {}".format("".join(
                [x + ', ' for x in cls.__allowed])))
        else:
            if t in ['i64', 'i32', 'i16', 'i8', 'u64', 'u32', 'u16', 'u8']:
                pytype = int
            elif t == 'PyString':
                pytype = str
            elif t == 'PyBool':
                pytype == bool
            new = type(t, (HashableTypeABC,), {
                       '_type': t, '_pytype': pytype, '__doc__': cls._doc})
            return new

    @classmethod
    def __subclasshook__(cls, C):
        if cls is HashableTypeABC:
            if hasattr(C, '_type') and C._type in cls.__allowed:
                return True
        else:
            return False

    def ishashable(other):
        if issubclass(other, str):
            return True
        elif issubclass(other, bool):
            return True
        elif issubclass(other, int):
            return True
        else:
            return False

HashableType = HashableTypeABC('HashableType', (HashableTypeABC,), {
                               "__doc__": HashableTypeABC._doc})


class PyDict(PythonObject):

    def __init__(self, ptr, signature, call_fn=None):
        self._ptr = ptr
        if not signature:
            raise MissingTypeHint(
                "rustypy: missing type hint for PyList unpacking in Python")
        self.sig = signature
        key_t = self.sig.__args__[0]
        if not issubclass(key_t, HashableTypeABC):
            TypeError("rustypy: the type corresponding to the key of a \
            dictionary must be a subclass of rustypy.HashableType")
        self.call_fn = call_fn

    def free(self):
        c_backend.pydict_free(self._ptr, self.key_rs_type)
        delattr(self, '_ptr')
        setattr(self, 'to_dict', _dangling_pointer)

    def to_dict(self, depth=0):
        key_t = self.sig.__args__[0]._type
        arg_t = self.sig.__args__[1]
        key_rs_t, _, fnk, key_py_t = PyDict.get_key_type_info(key_t)
        drain_iter = c_backend.pydict_get_drain(self._ptr, key_rs_t)
        # get the functions for extracting the key and the value
        if arg_t is str:
            fnv = c_backend.pyarg_extract_owned_str
        elif arg_t is bool:
            fnv = c_backend.pyarg_extract_owned_bool
        elif arg_t is int:
            fnv = c_backend.pyarg_extract_owned_int
        elif arg_t is Double or arg_t is float:
            fnv = c_backend.pyarg_extract_owned_double
        elif arg_t is Float:
            fnv = c_backend.pyarg_extract_owned_float
        elif issubclass(arg_t, typing.Tuple):
            fnv = c_backend.pyarg_extract_owned_tuple
        elif issubclass(arg_t, typing.List):
            fnv = c_backend.pyarg_extract_owned_list
        elif issubclass(arg_t, typing.Dict):
            fnv = c_backend.pyarg_extract_owned_dict
        else:
            raise TypeError("rustypy: subtype `{t}` of Dict type is \
                            not supported".format(t=arg_t))
        pydict, kv_tuple = [], True
        kv_tpl_type = typing.Tuple[key_py_t, arg_t]
        # run the drain iterator while not a null pointer
        while kv_tuple:
            kv_tuple = c_backend.pydict_drain_element(
                drain_iter, key_rs_t)
            if not kv_tuple:
                break
            t = PyTuple(kv_tuple, kv_tpl_type, self.call_fn)
            pydict.append(t.to_tuple(depth + 1))
            t.free()
        return dict(pydict)

    @staticmethod
    def from_dict(source: dict, sig):
        key_t = sig.__args__[0]
        arg_t = sig.__args__[1]
        if not issubclass(key_t, HashableTypeABC):
            TypeError("rustypy: the type corresponding to the key of a \
            dictionary must be a subclass of rustypy.HashableType")
        key_rs_t, fnk, _, _ = PyDict.get_key_type_info(key_t._type)
        if arg_t is str:
            fnv = _to_pystring
        elif arg_t is bool:
            fnv = _to_pybool
        elif arg_t is int:
            fnv = c_backend.pyarg_from_int
        elif arg_t is Double or arg_t is float:
            fnv = c_backend.pyarg_from_double
        elif arg_t is Float:
            fnv = c_backend.pyarg_from_float
        elif issubclass(arg_t, typing.Tuple):
            fnv = _to_pytuple(arg_t)
        elif issubclass(arg_t, typing.List):
            fnv = _to_pylist(arg_t)
        elif issubclass(arg_t, typing.Dict):
            fnv = _to_pydict(arg_t)
        else:
            raise TypeError("rustypy: subtype {t} of List type is \
                            not supported".format(t=arg_t))
        pydict = c_backend.pydict_new(key_rs_t)
        for k, v in source.items():
            c_backend.pydict_insert(pydict, key_rs_t, fnk(k), fnv(v))
        return pydict

    @staticmethod
    def get_key_type_info(key_t):
        if key_t == 'i8':
            key_rs_t = c_backend.pydict_get_key_type(1)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'u8':
            key_rs_t = c_backend.pydict_get_key_type(2)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'i16':
            key_rs_t = c_backend.pydict_get_key_type(3)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'u16':
            key_rs_t = c_backend.pydict_get_key_type(4)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'i32':
            key_rs_t = c_backend.pydict_get_key_type(5)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'u32':
            key_rs_t = c_backend.pydict_get_key_type(6)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'i64':
            key_rs_t = c_backend.pydict_get_key_type(7)
            fnk = c_backend.pyarg_from_int
            fne = c_backend.pyarg_extract_owned_int
            key_py_t = int
        elif key_t == 'u64':
            key_rs_t = c_backend.pydict_get_key_type(8)
            fnk = c_backend.pyarg_from_ulonglong
            fne = c_backend.pyarg_extract_owned_ulonglong
            key_py_t = UnsignedLongLong
        elif key_t == 'PyBool':
            key_rs_t = c_backend.pydict_get_key_type(11)
            fnk = _to_pybool
            fne = c_backend.pyarg_extract_owned_bool
            key_py_t = bool
        elif key_t == 'PyString':
            key_rs_t = c_backend.pydict_get_key_type(12)
            fnk = _to_string
            fne = c_backend.pyarg_extract_owned_string
            key_py_t = str
        return (key_rs_t, fnk, fne, key_py_t)

    @property
    def key_rs_type(self):
        if not hasattr(self, '_key_rs_type'):
            rst, _, _, pyt = PyDict.get_key_type_info(
                self.sig.__args__[0]._type)
            self._key_rs_type = rst
            self._key_py_type = pyt
        return self._key_rs_type

    @property
    def key_py_type(self):
        if not hasattr(self, '_key_py_type'):
            rst, _, _, pyt = PyDict.get_key_type_info(
                self.sig.__args__[0]._type)
            self._key_rs_type = rst
            self._key_py_type = pyt
        return self._key_py_type

# ==================== #
#   Conversion Funcs   #
# ==================== #

FIND_TYPE = re.compile("type\((.*)\)")

RustType = namedtuple('RustType', ['equiv', 'ref', 'mutref'])


class RAW_POINTER(object):
    pass


def _get_signature_types(params):
    def inner_types(t):
        t = t.strip()
        mutref, ref = False, False
        if "&mut" in t or "*mut" in t:
            type_ = t.replace("&mut", '').replace("*mut", '').strip()
            mutref = True
        elif "&" in t or "*const" in t:
            type_ = t.replace('&', '').replace("*const", '').strip()
            ref = True
        else:
            type_ = t
        try:
            equiv = RS_TYPE_CONVERSION[type_]
        except:
            raise TypeError('rustypy: type not supported: {}'.format(type_))
        else:
            if equiv == 'int':
                return RustType(equiv=int, ref=ref, mutref=mutref)
            elif equiv == 'float':
                return RustType(equiv=Float, ref=ref, mutref=mutref)
            elif equiv == 'double':
                return RustType(equiv=Double, ref=ref, mutref=mutref)
            elif equiv == 'str':
                return RustType(equiv=str, ref=True, mutref=False)
            elif equiv == 'bool':
                return RustType(equiv=bool, ref=True, mutref=False)
            elif equiv == 'tuple':
                return RustType(equiv=tuple, ref=True, mutref=False)
            elif equiv == 'list':
                return RustType(equiv=list, ref=True, mutref=mutref)
            elif equiv == 'POINTER':
                return RustType(equiv=RAW_POINTER, ref=True, mutref=mutref)
            elif equiv == 'None':
                return RustType(equiv=None, ref=False, mutref=False)

    params = [x for x in params.split(';') if x != '']
    param_types = []
    for p in params:
        param_types.append(re.search(FIND_TYPE, p).group(1))
        param_types[-1] = inner_types(param_types[-1])
    return param_types


def _get_ptr_to_C_obj(obj, sig=None):
    if isinstance(obj, bool):
        return PyBool.from_bool(obj)
    elif isinstance(obj, int):
        return ctypes.c_longlong(obj)
    elif isinstance(obj, Float):
        return ctypes.c_float(obj)
    elif isinstance(obj, Double) or isinstance(obj, float):
        return ctypes.c_double(obj)
    elif isinstance(obj, str):
        return PyString.from_str(obj)
    elif isinstance(obj, tuple):
        if not sig:
            raise MissingTypeHint(
                "rustypy: tuple type arguments require a type hint")
        return PyTuple.from_tuple(obj, sig)
    elif isinstance(obj, list):
        if not sig:
            raise MissingTypeHint(
                "rustypy: list type arguments require a type hint")
        return PyList.from_list(obj, sig)
    elif isinstance(obj, dict):
        if not sig:
            raise MissingTypeHint(
                "rustypy: dict type arguments require a type hint")
        if not issubclass(sig, dict):
            raise TypeError(
                "rustypy: the type hint must be of typing.Dict type")
        return PyDict.from_dict(obj, sig)
    elif isinstance(obj, RAW_POINTER):
        if not sig:
            raise MissingTypeHint(
                "rustypy: raw pointer type arguments require type information \
                 for proper conversion")
        raise NotImplementedError


def _extract_pytypes(ref, sig=False, call_fn=None, depth=0, elem_num=None):
    if isinstance(ref, int):
        return ref
    elif isinstance(ref, float):
        return ref
    elif isinstance(ref, POINTER(ctypes.c_longlong)):
        return ref.contents
    elif isinstance(ref, POINTER(ctypes.c_float)):
        return ref.contents
    elif isinstance(ref, POINTER(ctypes.c_double)):
        return ref.contents
    elif isinstance(ref, POINTER(PyTuple_RS)):
        pyobj = PyTuple(ref, sig, call_fn=call_fn)
        val = pyobj.to_tuple(depth)
        if depth == 0:
            pyobj.free()
        return val
    elif isinstance(ref, POINTER(PyString_RS)):
        pyobj = PyString(ref)
        val = pyobj.to_str()
        return val
    elif isinstance(ref, POINTER(PyBool_RS)):
        pyobj = PyBool(ref)
        val = pyobj.to_bool()
        if depth == 0:
            pyobj.free()
        return val
    elif isinstance(ref, POINTER(PyList_RS)):
        pyobj = PyList(ref, sig, call_fn=call_fn)
        val = pyobj.to_list(depth)
        if depth == 0:
            pyobj.free()
        return val
    elif isinstance(ref, POINTER(PyDict_RS)):
        pyobj = PyDict(ref, sig, call_fn=call_fn)
        val = pyobj.to_dict(depth)
        if depth == 0:
            pyobj.free()
        return val
    elif isinstance(ref, POINTER(Raw_RS)):
        raise NotImplementedError
    else:
        raise TypeError("rustypy: return type not supported")

# ============================= #
#   Helper classes and funcs    #
# ============================= #


def get_crate_entry(mod, manifest):
    rgx_lib = re.compile(r'\[lib\]')
    rgx_path = re.compile(r'path(\W+|)=(\W+|)[\'\"](?P<entry>.*)[\'\"]')
    inlibsection, entry = False, None
    with open(manifest, 'r') as f:
        for l in f:
            if inlibsection:
                entry = re.match(rgx_path, l)
                if entry:
                    entry = entry.group('entry')
                    entry = os.path.join(*entry.split('/'))
                    break
            elif not inlibsection and re.search(rgx_lib, l):
                inlibsection = True
    if not entry:
        entry = os.path.join('src', 'lib.rs')
    return os.path.join(mod, entry)


def bind_rs_crate_funcs(mod, lib, prefixes=None):
    if not c_backend:
        load_rust_lib()
    if not os.path.exists(mod):
        raise OSError('rustypy: the specified `{}` Rust crate does not exist')
    elif not os.path.exists(lib):
        raise OSError(
            'rustypy: the specified `{}` compiled library file does not exist')
    manifest = os.path.join(mod, 'Cargo.toml')
    if not os.path.exists(manifest):
        raise OSError("rustypy: no Cargo(.toml) manifest found for this crate")
    entry_point = get_crate_entry(mod, manifest)
    return RustBinds(entry_point, lib, prefixes=prefixes)


class KrateData(object):

    def __init__(self, prefixes):
        self.obj = c_backend.krate_data_new(prefixes)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        c_backend.krate_data_free(self.obj)

    def __iter__(self):
        self._idx = 0
        self._len = c_backend.krate_data_len(self.obj)
        return self

    def __next__(self):
        if (self._len - 1) == -1 or self._idx > (self._len - 1):
            self._idx = 0
            raise StopIteration
        val = c_backend.krate_data_iter(self.obj, self._idx)
        self._idx += 1
        return PyString(val)


class RustBinds(object):
    """Main binding generator class."""

    def __init__(self, entry_point, compiled_lib, prefixes=None):
        self._FFI = ctypes.cdll.LoadLibrary(compiled_lib)
        if isinstance(prefixes, str):
            p = [prefixes]
        elif isinstance(prefixes, list):
            p = prefixes
        else:
            p = ["python_bind_"]
        p = PyList.from_list(p, typing.List[str])
        self._krate_data = KrateData(p)
        entry = PyString.from_str(entry_point)
        signal = c_backend.parse_src(entry, self._krate_data.obj)
        if signal == 1:
            raise Exception(
                "rustypy: failed to generate Rust bindings, the source "
                "code didn't parse, checkout if your library compiles!")
        prepared_funcs = {}
        with self._krate_data as krate:
            for e in krate:
                decl = e.to_str()
                if decl == "NO_IDX_ERROR":
                    break
                for prefix in prefixes:
                    s = decl.split(prefix)
                    if len(s) == 2:
                        name, decl = s[0], s[1]
                        break
                name, params = decl.split('::', maxsplit=1)
                name = prefix + name
                params = _get_signature_types(params)
                fn = getattr(self._FFI, "{}".format(name))
                RustBinds.decl_C_args(fn, params)
                prepared_funcs[name] = self.FnCall(name, params, self._FFI)
        for name, fn in prepared_funcs.items():
            setattr(self, name, fn)

    class FnCall(object):

        def __init__(self, name, argtypes, lib):
            self._rs_fn = getattr(lib, name)
            self._fn_name = name
            self.__type_hints = {'real_return': argtypes.pop()}
            self.__type_hints['real_argtypes'] = argtypes

        def __call__(self, *args, **kwargs):
            if kwargs:
                return_ref = kwargs.get('return_ref')
                get_contents = kwargs.get('get_contents')
            else:
                return_ref = False
                get_contents = False
            n_args = len(self.argtypes)
            g_args = len(args)
            if g_args != n_args:
                raise TypeError("rustypy: {}() takes exactly {} "
                                "arguments ({} given)".format(
                                    self._fn_name, n_args, g_args))
            prep_args = []
            for x, a in enumerate(args):
                p = self.argtypes[x]
                if p.ref or p.mutref:
                    sig = self.get_argtype(x)
                    ref = _get_ptr_to_C_obj(a, sig=sig)
                    prep_args.append(ref)
                elif isinstance(a, bool):
                    ref = _get_ptr_to_C_obj(a)
                    prep_args.append(ref)
                elif isinstance(a, str):
                    ref = _get_ptr_to_C_obj(a)
                    prep_args.append(ref)
                elif isinstance(a, int) or isinstance(a, float):
                    prep_args.append(a)
                else:
                    raise TypeError("rustypy: argument #{} type of `{}` passed to "
                                    "function `{}` not supported".format(
                                        x, a, self._fn_name))
            result = self._rs_fn(*prep_args)
            if not return_ref:
                try:
                    python_result = _extract_pytypes(
                        result, call_fn=self, sig=self.restype)
                except MissingTypeHint:
                    raise TypeError("rustypy: must add return type of "
                                    "function `{}`".format(self._fn_name))
                return python_result
            elif get_contents:
                arg_refs = []
                for x, r in enumerate(prep_args):
                    if isinstance(r, POINTER(PyString_RS)):
                        arg_refs.append(f_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(PyBool_RS)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(ctypes.c_longlong)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(ctypes.c_float)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(ctypes.c_double)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(PyTuple_RS)):
                        arg_refs.append(_extract_pytypes(
                            r, call_fn=self, sig=self.get_argtype(x)))
                    elif isinstance(r, POINTER(PyList_RS)):
                        arg_refs.append(_extract_pytypes(
                            r, call_fn=self, sig=self.get_argtype(x)))
                    elif isinstance(r, POINTER(PyDict_RS)):
                        arg_refs.append(_extract_pytypes(
                            r, call_fn=self, sig=self.get_argtype(x)))
                    else:
                        arg_refs.append(r.value)
                return result, arg_refs
            else:
                arg_refs = []
                for x, r in enumerate(prep_args):
                    arg_refs.append(r)
                return result, arg_refs

        @property
        def real_restype(self):
            return self.__type_hints['real_return']

        @property
        def restype(self):
            try:
                return self.__type_hints['return']
            except KeyError:
                return

        @restype.setter
        def restype(self, annotation):
            self.__type_hints['return'] = annotation
            if issubclass(annotation, dict):
                real_t = self.__type_hints['real_return']
                self.__type_hints['real_return'] = RustType(
                    equiv=dict, ref=True, mutref=real_t.mutref)
                r_args = [x for x in self.__type_hints['real_argtypes']]
                r_args.append(self.real_restype)
                RustBinds.decl_C_args(self._rs_fn, r_args)

        @property
        def argtypes(self):
            try:
                return self.__type_hints['real_argtypes']
            except KeyError:
                return

        @argtypes.setter
        def argtypes(self):
            raise AttributeError(
                "rustypy: private attribute, cannot be set directly")

        def add_argtype(self, position, hint):
            types = self.__type_hints.setdefault(
                'argtypes', [None] * len(self.argtypes))
            real_t = self.argtypes[position]
            if real_t.equiv is list \
                    and not issubclass(hint, typing.List):
                raise TypeError("rustypy: type hint for argument {n} of function {fn} \
                must be of typing.List type")
            elif real_t.equiv is RAW_POINTER:
                if issubclass(hint, dict):
                    self.__type_hints['real_argtypes'][position] = RustType(
                        equiv=dict, ref=True, mutref=real_t.mutref)
                    r_args = [x for x in self.__type_hints['real_argtypes']]
                    r_args.append(self.real_restype)
                    RustBinds.decl_C_args(self._rs_fn, r_args)
            types[position] = hint

        def get_argtype(self, position):
            hints = self.__type_hints.get('argtypes')
            if hints:
                return hints[position]

    @staticmethod
    def decl_C_args(FFI, params):
        restype = None
        argtypes = []
        for x, p in enumerate(params, 1):
            if p.equiv is None:
                add_p = c_void_p
            elif issubclass(p.equiv, bool):
                add_p = PyBool_RS
            elif issubclass(p.equiv, int):
                add_p = ctypes.c_longlong
            elif issubclass(p.equiv, float):
                add_p = p.equiv._definition
            elif issubclass(p.equiv, str):
                add_p = PyString_RS
            elif issubclass(p.equiv, tuple):
                add_p = PyTuple_RS
            elif issubclass(p.equiv, list):
                add_p = PyList_RS
            elif issubclass(p.equiv, dict):
                add_p = PyDict_RS
            elif issubclass(p.equiv, RAW_POINTER):
                add_p = Raw_RS
            if p.mutref or p.ref:
                add_p = POINTER(add_p)
            if x <= (len(params) - 1):
                argtypes.append(add_p)
            else:
                restype = add_p
        setattr(FFI, "restype", restype)
        if len(argtypes) > 0:
            setattr(FFI, "argtypes", tuple(argtypes))

# WIP:


class RsStruct(object):
    """
    Example usage:
    binds = RustBinds()
    new_foo = binds.foo_struct()
    new_foo.method_call()
    """

    class NoConstructor(AttributeError):

        def __init__(self, name, mod):
            self.name = name

        def __str__(self, name, mod):
            msg = "rustypy: `new` (constructor) method name not defined for struct `{}`" \
                " in module `{}`"
            msg = msg.format(self.name)
            return msg

    class StructPtr(object):
        _ERR_RESERVED = "rustypy: cannot use `krate` attr name, is a reserved attribute"

        def __init__(self, kls, ffi, krate):
            self.__krate = krate
            self.ffi = ffi

        @property
        def krate(self):
            raise AttributeError(self._ERR_RESERVED)

        @krate.setter
        def prt(self, val):
            raise AttributeError(self._ERR_RESERVED)

        @krate.deleter
        def prt(self, val):
            raise AttributeError(self._ERR_RESERVED)

    def __init__(self, ffi, method_list):
        self.ffi = ffi
        for m in method_list:
            __add_method(m)

    def __add_method(self, method):
        params = get_signature_types(method)
        # new_method is a staticmethod
        setattr(self, method.name, new_method)

    def get_mod(self):
        pass

    def __call__(self):
        try:
            krate = self.new()
        except AttributeError:
            raise NoConstructor(self.name, self.get_mod())
        return StructPtr(self, ffi, krate)


class ModuleKlass(object):

    def __init__(self):
        pass

    def parse_struct(self, module):
        pass

    def add_child_mod(self, mod):
        setattr(self, mod.name, mod)

    def add_child_func(self, func):
        setattr(self, func.name, struct)

    def add_child_struct(self, struct):
        setattr(self, struct.name, struct)
