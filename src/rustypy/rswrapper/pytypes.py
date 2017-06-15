"""PyTypes wrappers."""

import abc
import ctypes
import typing

from collections import deque, namedtuple

global c_backend
from .ffi_defs import *
c_backend = get_rs_lib()


class MissingTypeHint(TypeError):
    pass


def _dangling_pointer(*args, **kwargs):
    raise ReferenceError(
        "rustypy: the underlying Rust type has been dropped")


class PythonObject(object):

    def __init__(self, ptr):
        self._ptr = ptr

    def get_rs_obj(self):
        return self._ptr


class PyString(PythonObject):

    def free(self):
        c_backend.pystring_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_str', _dangling_pointer)

    def to_str(self):
        """Consumes the wrapper and returns a Python string.
        Afterwards is not necessary to destruct it as it has already
        been consumed."""
        val = c_backend.pystring_get_str(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_str', _dangling_pointer)
        return val.decode("utf-8")

    @staticmethod
    def from_str(s: str):
        return c_backend.pystring_new(s.encode("utf-8"))


class PyBool(PythonObject):

    def free(self):
        c_backend.pybool_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_bool', _dangling_pointer)

    def to_bool(self):
        val = c_backend.pybool_get_val(self._ptr)
        if val == 0:
            val = False
        else:
            val = True
        self.free()
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


def _to_pytuple(signature):
    def dec(arg):
        return c_backend.pyarg_from_pytuple(PyTuple.from_tuple(arg, signature))
    return dec


def _to_pylist(signature):
    def dec(arg):
        return c_backend.pyarg_from_pylist(PyList.from_list(arg, signature))
    return dec


def _to_pydict(signature):
    def dec(arg):
        d = PyDict.from_dict(arg, signature)
        return c_backend.pyarg_from_pydict(d)
    return dec


def _extract_value(pyarg, arg_t, depth=0):
    pytype = None
    if arg_t is str:
        content = c_backend.pyarg_extract_owned_str(pyarg)
        pytype = c_backend.pystring_get_str(content).decode("utf-8")
    elif arg_t is bool:
        b = PyBool(c_backend.pyarg_extract_owned_bool(pyarg))
        pytype = b.to_bool()
    elif arg_t is int:
        pytype = c_backend.pyarg_extract_owned_int(pyarg)
    elif arg_t is UnsignedLongLong:
        pytype = c_backend.pyarg_extract_owned_ulonglong(pyarg)
    elif arg_t is Double or arg_t is float:
        pytype = c_backend.pyarg_extract_owned_double(pyarg)
    elif arg_t is Float:
        pytype = c_backend.pyarg_extract_owned_float(pyarg)
    elif issubclass(arg_t, Tuple):
        ptr = c_backend.pyarg_extract_owned_tuple(pyarg)
        t = PyTuple(ptr, arg_t)
        pytype = t.to_tuple(depth=depth + 1)
    elif issubclass(arg_t, typing.List):
        ptr = c_backend.pyarg_extract_owned_list(pyarg)
        l = PyList(ptr, arg_t)
        pytype = l.to_list(depth=depth + 1)
    elif issubclass(arg_t, typing.Dict):
        ptr = c_backend.pyarg_extract_owned_dict(pyarg)
        d = PyDict(ptr, arg_t)
        pytype = d.to_dict(depth=depth + 1)
    return pytype


class PyTuple(PythonObject):

    def __init__(self, ptr, signature, call_fn=None):
        self._ptr = ptr
        if not signature:
            raise MissingTypeHint(
                "rustypy: missing type hint for PyTuple unpacking in Python")
        if not issubclass(signature, Tuple):
            raise TypeError("rustypy: expecting rustypy Tuple definition, found `{}` instead"
                            .format(signature))
        self.signature = signature
        self.call_fn = call_fn

    def free(self):
        c_backend.pytuple_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_tuple', _dangling_pointer)

    def to_tuple(self, depth=0):
        arity = c_backend.pytuple_len(self._ptr)
        if arity != len(self.signature) and self.call_fn:
            raise TypeError("rustypy: the type hint for returning tuple of fn `{}` "
                            "and the return tuple value are not of "
                            "the same length".format(self.call_fn._fn_name))
        elif arity != len(self.signature):
            raise TypeError(
                "rustypy: type hint for PyTuple is of wrong length")
        tuple_elems = []
        for pos, arg_t in enumerate(self.signature):
            pyarg = c_backend.pytuple_get_element(self._ptr, pos)
            pytype = _extract_value(pyarg, arg_t, depth=depth + 1)
            if pytype is None:
                raise TypeError("rustypy: subtype `{t}` of Tuple type is "
                                "not supported".format(t=arg_t))
            tuple_elems.append(pytype)
        self.free()
        return tuple(tuple_elems)

    @staticmethod
    def from_tuple(source: tuple, signature):
        try:
            if not issubclass(signature, Tuple):
                raise Exception
        except:
            raise TypeError("rustypy: type hint for PyTuple.from_tuple "
                            "must be of rustypy.Tuple type")
        next_e = None
        cnt = len(source) - 1
        for i in range(0, len(source)):
            cnt = cnt - i
            arg_t = signature._element_type(cnt)
            element = source[cnt]
            if arg_t is str:
                pyarg = _to_pystring(element)
            elif arg_t is bool:
                pyarg = _to_pystring(element)
            elif arg_t is int:
                pyarg = c_backend.pyarg_from_int(element)
            elif arg_t is Double or arg_t is float:
                pyarg = c_backend.pyarg_from_double(element)
            elif arg_t is Float:
                pyarg = c_backend.pyarg_from_float(element)
            elif issubclass(arg_t, Tuple):
                pyarg = _to_pytuple(arg_t)(element)
            elif issubclass(arg_t, list):
                pyarg = _to_pylist(arg_t)(element)
            elif issubclass(arg_t, dict):
                pyarg = _to_pydict(arg_t)(element)
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
        self.signature = signature
        self.call_fn = call_fn

    def free(self):
        c_backend.pylist_free(self._ptr)
        delattr(self, '_ptr')
        setattr(self, 'to_list', _dangling_pointer)

    def to_list(self, depth=0):
        arg_t = self.signature.__args__[0]
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
        elif issubclass(arg_t, Tuple):
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                ptr = c_backend.pyarg_extract_owned_tuple(pyarg)
                t = PyTuple(ptr, arg_t)
                pylist.appendleft(t.to_tuple(depth=depth + 1))
                last -= 1
        elif issubclass(arg_t, typing.List):
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                ptr = c_backend.pyarg_extract_owned_list(pyarg)
                l = PyList(ptr, arg_t)
                pylist.appendleft(l.to_list(depth=depth + 1))
                last -= 1
        elif issubclass(arg_t, typing.Dict):
            for e in range(0, self._len):
                pyarg = c_backend.pylist_get_element(self._ptr, last)
                ptr = c_backend.pyarg_extract_owned_dict(pyarg)
                d = PyDict(ptr, arg_t)
                pylist.appendleft(d.to_dict(depth=depth + 1))
                last -= 1
        else:
            raise TypeError("rustypy: subtype `{t}` of List type is \
                            not supported".format(t=arg_t))
        self.free()
        return list(pylist)

    @staticmethod
    def from_list(source: list, signature):
        arg_t = signature.__args__[0]
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
        elif issubclass(arg_t, Tuple):
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
        self.signature = signature
        key_t = self.signature.__args__[0]
        if not issubclass(key_t, HashableTypeABC):
            TypeError("rustypy: the type corresponding to the key of a \
            dictionary must be a subclass of rustypy.HashableType")
        self.call_fn = call_fn

    def free(self):
        c_backend.pydict_free(self._ptr, self.key_rs_type)
        delattr(self, '_ptr')
        setattr(self, 'to_dict', _dangling_pointer)

    def to_dict(self, depth=0):
        key_t = self.signature.__args__[0]._type
        arg_t = self.signature.__args__[1]
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
        elif issubclass(arg_t, Tuple):
            fnv = c_backend.pyarg_extract_owned_tuple
        elif issubclass(arg_t, typing.List):
            fnv = c_backend.pyarg_extract_owned_list
        elif issubclass(arg_t, typing.Dict):
            fnv = c_backend.pyarg_extract_owned_dict
        else:
            raise TypeError("rustypy: subtype `{t}` of Dict type is \
                            not supported".format(t=arg_t))
        pydict, kv_tuple = [], True
        # run the drain iterator while not a null pointer
        while kv_tuple:
            kv_tuple = c_backend.pydict_drain_element(
                drain_iter, key_rs_t)
            if not kv_tuple:
                break
            key = c_backend.pydict_get_kv(0, kv_tuple)
            val = c_backend.pydict_get_kv(1, kv_tuple)
            t = (fnk(key),
                 _extract_value(val, arg_t))
            c_backend.pydict_free_kv(kv_tuple)
            pydict.append(t)
        self.free()
        return dict(pydict)

    @staticmethod
    def from_dict(source: dict, signature):
        key_t = signature.__args__[0]
        arg_t = signature.__args__[1]
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
        elif issubclass(arg_t, Tuple):
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
            fnk = _to_pystring
            fne = c_backend.pyarg_extract_owned_string
            key_py_t = str
        return (key_rs_t, fnk, fne, key_py_t)

    @property
    def key_rs_type(self):
        if not hasattr(self, '_key_rs_type'):
            rst, _, _, pyt = PyDict.get_key_type_info(
                self.signature.__args__[0]._type)
            self._key_rs_type = rst
            self._key_py_type = pyt
        return self._key_rs_type

    @property
    def key_py_type(self):
        if not hasattr(self, '_key_py_type'):
            rst, _, _, pyt = PyDict.get_key_type_info(
                self.signature.__args__[0]._type)
            self._key_rs_type = rst
            self._key_py_type = pyt
        return self._key_py_type

from .rswrapper import Float, Double, UnsignedLongLong, Tuple
