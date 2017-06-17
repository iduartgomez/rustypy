# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python."""

import os.path
import re
import types
import typing
import inspect
from collections import deque, namedtuple
from string import Template

from .ffi_defs import *
from .ffi_defs import get_rs_lib
from .pytypes import MissingTypeHint, PyBool, PyDict, PyList, PyString, PyTuple

global c_backend
c_backend = get_rs_lib()

# ==================== #
#   Conversion Funcs   #
# ==================== #

FIND_TYPE = re.compile("type\((.*)\)")

RustType = namedtuple('RustType', ['equiv', 'ref', 'mutref', 'raw'])

Float = type('Float', (float,), {'_definition': ctypes.c_float})
Double = type('Double', (float,), {'_definition': ctypes.c_double})
UnsignedLongLong = type('UnsignedLongLong', (int,), {
                        '_definition': ctypes.c_ulonglong})


class TupleMeta(type):

    def __new__(cls, name, bases, namespace, parameters=None):
        self = super().__new__(cls, name, bases, namespace)
        self.__iter_cnt = 0
        if not parameters:
            self.__params = None
            return self
        self.__params = []
        for arg_t in parameters:
            if arg_t is str:
                self.__params.append(str)
            elif arg_t is bool:
                self.__params.append(bool)
            elif arg_t is int:
                self.__params.append(int)
            elif arg_t is UnsignedLongLong:
                self.__params.append(UnsignedLongLong)
            elif arg_t is Double or arg_t is float:
                self.__params.append(Double)
            elif arg_t is Float:
                self.__params.append(Float)
            elif issubclass(arg_t, Tuple):
                self.__params.append(arg_t)
            elif issubclass(arg_t, list):
                self.__params.append(arg_t)
            elif issubclass(arg_t, dict):
                self.__params.append(arg_t)
            elif arg_t.__class__ is typing.GenericMeta:
                self.__params.append(arg_t)
            else:
                raise TypeError("rustypy: subtype `{t}` of Tuple type is \
                                not supported".format(t=arg_t))
        return self

    def __init__(self, *args, **kwds):
        pass

    def __len__(self):
        return len(self.__params)

    def __getitem__(self, parameters):
        if self.__params is not None:
            raise TypeError("Cannot re-parameterize %r" % (self,))
        if not isinstance(parameters, tuple):
            parameters = (parameters,)
        return self.__class__(self.__name__, self.__bases__, dict(self.__dict__), parameters)

    def __repr__(self):
        if self.__params is None:
            return "rutypy.Tuple"
        inner = "".join(["%r, " % e if i + 1 < len(self.__params) else repr(e)
                         for i, e in enumerate(self.__params)])
        return "rustypy.Tuple[{}]".format(inner)

    def _element_type(self, pos):
        return self.__params[pos]

    def __subclasscheck__(self, cls):
        if cls is typing.Any:
            return true
        if isinstance(cls, tuple):
            return True
        if not isinstance(cls, TupleMeta):
            return False
        else:
            return True

    def __iter__(self):
        return self

    def __next__(self):
        if not self.__params:
            return StopIteration()
        if self.__iter_cnt < len(self.__params):
            e = self.__params[self.__iter_cnt]
            self.__iter_cnt += 1
            return e
        else:
            self.__iter_cnt = 0
            raise StopIteration()


class Tuple(metaclass=TupleMeta):
    __slots__ = ()

    def __new__(self, *args, **kwds):
        raise TypeError("Cannot subclass %r" % (self,))


class OpaquePtr(object):
    pass


def _get_signature_types(params):
    def inner_types(t):
        t = t.strip()
        mutref, ref, raw = False, False, False
        if "&mut" in t:
            type_ = t.replace("&mut", '').strip()
            mutref = True
        elif "*mut" in t:
            type_ = t.replace("*mut", '').strip()
            mutref, raw = True, True
        elif "&" in t:
            type_ = t.replace('&', '').strip()
            ref = True
        elif "*const" in t:
            type_ = t.replace("*const", '').strip()
            ref, raw = True, True
        else:
            type_ = t
        try:
            equiv = RS_TYPE_CONVERSION[type_]
        except:
            raise TypeError("rustypy: type not supported: {}".format(type_))
        else:
            if equiv == 'int':
                return RustType(equiv=int, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'float':
                return RustType(equiv=Float, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'double':
                return RustType(equiv=Double, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'str':
                return RustType(equiv=str, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'bool':
                return RustType(equiv=bool, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'tuple':
                return RustType(equiv=tuple, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'list':
                return RustType(equiv=list, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'OpaquePtr':
                return RustType(equiv=OpaquePtr, ref=ref, mutref=mutref, raw=raw)
            elif equiv == 'None':
                return RustType(equiv=None, ref=False, mutref=False, raw=False)

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
    elif isinstance(obj, OpaquePtr):
        if not sig:
            raise MissingTypeHint(
                "rustypy: raw pointer type arguments require type information \
                 for proper type coercion")
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
        return val
    elif isinstance(ref, POINTER(PyString_RS)):
        pyobj = PyString(ref)
        val = pyobj.to_str()
        return val
    elif isinstance(ref, POINTER(PyBool_RS)):
        pyobj = PyBool(ref)
        val = pyobj.to_bool()
        return val
    elif isinstance(ref, POINTER(PyList_RS)):
        pyobj = PyList(ref, sig, call_fn=call_fn)
        val = pyobj.to_list(depth)
        return val
    elif isinstance(ref, POINTER(PyDict_RS)):
        pyobj = PyDict(ref, sig, call_fn=call_fn)
        val = pyobj.to_dict(depth)
        return val
    elif isinstance(ref, POINTER(Raw_RS)):
        raise NotImplementedError
    else:
        raise TypeError("rustypy: return type not supported")

# ============================= #
#   Helper classes and funcs    #
# ============================= #


def get_crate_entry(mod):
    manifest = os.path.join(mod, 'Cargo.toml')
    if os.path.exists(manifest):
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
    else:
        if os.path.isfile(mod) and os.path.basename(mod).endswith('.rs'):
            return mod
        else:
            default = os.path.join(mod, 'lib.rs')
            if os.path.exists(default):
                return default
            else:
                raise OSError(
                    "rustypy: couldn't find lib.rs in the specified directory")


def bind_rs_crate_funcs(mod, lib, prefixes=None):
    if not c_backend:
        load_rust_lib()
    if not os.path.exists(mod):
        raise OSError('rustypy: `{}` path does not exist')
    elif not os.path.exists(lib):
        raise OSError(
            'rustypy: `{}` compiled library file does not exist')
    entry_point = get_crate_entry(mod)
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
        ret_msg = c_backend.parse_src(entry, self._krate_data.obj)
        if ret_msg:
            raise Exception(
                "rustypy: failed to generate Rust bindings, failed with error:\n"
                "{}".format(PyString(ret_msg).to_str()))
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
                    equiv=dict, ref=real_t.ref, mutref=real_t.mutref, raw=real_t.raw)
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
            elif real_t.equiv is OpaquePtr:
                if issubclass(hint, dict):
                    self.__type_hints['real_argtypes'][position] = RustType(
                        equiv=dict, ref=real_t.ref, mutref=real_t.mutref, raw=real_t.raw)
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
            elif issubclass(p.equiv, OpaquePtr):
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
