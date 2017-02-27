# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python."""

import re
import types
import typing

from string import Template
from collections import deque, namedtuple

from .ffi_defs import *

global c_backend
from .ffi_defs import get_rs_lib
c_backend = get_rs_lib()

# ==================== #
#   Conversion Funcs   #
# ==================== #

FIND_TYPE = re.compile("type\((.*)\)")

RustType = namedtuple('RustType', ['equiv', 'ref', 'mutref'])

Float = type('Float', (float,), {'_definition': ctypes.c_float})
Double = type('Double', (float,), {'_definition': ctypes.c_double})
UnsignedLongLong = type('UnsignedLongLong', (int,), {
                        '_definition': ctypes.c_ulonglong})


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

from .pytypes import PyString, PyBool, PyTuple, PyList, PyDict
from .pytypes import MissingTypeHint
