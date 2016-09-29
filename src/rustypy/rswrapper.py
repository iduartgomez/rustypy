# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python."""

import os
import sys
import pkg_resources
import re
import types
import typing
from string import Template

##### CFFI #####
import ctypes
from ctypes import POINTER
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
    'void': 'None',
    'Vec': 'list',
    'HashMap': 'dict',
}


class PyString_RS(ctypes.Structure):
    pass


class PyBool_RS(ctypes.Structure):
    pass


class PyTuple_RS(ctypes.Structure):
    pass


class KrateData_RS(ctypes.Structure):
    pass


def config_ctypes():
    # Krate parsing functions
    c_backend.krate_data_new.restype = POINTER(KrateData_RS)
    c_backend.krate_data_free.argtypes = (POINTER(KrateData_RS), )
    c_backend.krate_data_free.restype = ctypes.c_void_p
    c_backend.krate_data_len.argtypes = (POINTER(KrateData_RS), )
    c_backend.krate_data_len.restype = ctypes.c_size_t
    c_backend.krate_data_iter.argtypes = (
        POINTER(KrateData_RS), ctypes.c_size_t)
    c_backend.krate_data_iter.restype = POINTER(PyString_RS)
    c_backend.parse_src.argtypes = (
        POINTER(PyString_RS), POINTER(KrateData_RS))
    c_backend.parse_src.restype = ctypes.c_uint

    # String related functions
    c_backend.PyString_new.argtypes = (ctypes.c_char_p, )
    c_backend.PyString_new.restype = POINTER(PyString_RS)
    c_backend.PyString_free.argtypes = (POINTER(PyString_RS), )
    c_backend.PyString_free.restype = ctypes.c_void_p
    c_backend.PyString_get_str.argtypes = (POINTER(PyString_RS), )
    c_backend.PyString_get_str.restype = ctypes.c_void_p

    # Bool related functions
    c_backend.PyBool_new.argtypes = (ctypes.c_byte, )
    c_backend.PyBool_new.restype = POINTER(PyBool_RS)
    c_backend.PyBool_free.argtypes = (POINTER(PyBool_RS), )
    c_backend.PyBool_free.restype = ctypes.c_void_p
    c_backend.PyBool_get_val.argtypes = (POINTER(PyBool_RS), )
    c_backend.PyBool_get_val.restype = ctypes.c_byte

    # Tuple related functions
    c_backend.PyTuple_len.argtypes = (POINTER(PyTuple_RS),)
    c_backend.PyTuple_len.restype = ctypes.c_size_t
    c_backend.PyTuple_extractPyInt.argtypes = (
        POINTER(PyTuple_RS), ctypes.c_size_t)
    c_backend.PyTuple_extractPyInt.restype = ctypes.c_longlong
    c_backend.PyTuple_extractPyFloat.argtypes = (
        POINTER(PyTuple_RS), ctypes.c_size_t)
    c_backend.PyTuple_extractPyFloat.restype = ctypes.c_float
    c_backend.PyTuple_extractPyDouble.argtypes = (
        POINTER(PyTuple_RS), ctypes.c_size_t)
    c_backend.PyTuple_extractPyDouble.restype = ctypes.c_double
    c_backend.PyTuple_extractPyBool.argtypes = (
        POINTER(PyTuple_RS), ctypes.c_size_t)
    c_backend.PyTuple_extractPyBool.restype = POINTER(PyBool_RS)
    c_backend.PyTuple_extractPyString.argtypes = (
        POINTER(PyTuple_RS), ctypes.c_size_t)
    c_backend.PyTuple_extractPyString.restype = POINTER(PyString_RS)


def load_rust_lib(recmpl=False):
    ext = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
    pre = {'win32': ''}.get(sys.platform, 'lib')
    lib = pkg_resources.resource_filename(
        'rslib', "{}rustypy{}".format(pre, ext))
    if not os.path.exists(lib) or recmpl:
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

if not c_backend or not rslib:
    load_rust_lib()


# ==================== #
#   Type Wrappers      #
# ==================== #

from collections import namedtuple
RustType = namedtuple('RustType', ['equiv', 'ref', 'mutref'])

Float = type('Float', (float,), {'_definition': ctypes.c_float})
Double = type('Double', (float,), {'_definition': ctypes.c_double})


class MissingTypeHint(TypeError):
    pass


class PythonObject(object):

    def __init__(self, ptr):
        self._ptr = ptr

    def get_rs_obj(self):
        return self._ptr


class PyString(PythonObject):

    def free(self):
        c_backend.PyString_free(self._ptr)

    def to_str(self):
        """Consumes the wrapper and returns a raw c_char pointer.
        Afterwards is not necessary to destruct it as it has already
        been consumed."""
        val = c_backend.PyString_get_str(self._ptr)
        pystr = ctypes.cast(val, ctypes.c_char_p).value.decode("utf-8")
        return pystr

    @staticmethod
    def from_str(s: str):
        return c_backend.PyString_new(s.encode("utf-8"))


class PyBool(PythonObject):

    def free(self):
        c_backend.PyBool_free(self._ptr)

    def to_bool(self):
        val = c_backend.PyBool_get_val(self._ptr)
        if val == 0:
            val = False
        else:
            val = True
        return val

    @staticmethod
    def from_bool(val: bool):
        if val is True:
            return c_backend.PyBool_new(1)
        else:
            return c_backend.PyBool_new(0)


class PyTuple(PythonObject):

    def __init__(self, ptr, signature, call_fn=None):
        self._ptr = ptr
        self.sig = signature
        self.call_fn = call_fn

    def free(self):
        c_backend.PyTuple_free(self._ptr)

    def to_tuple(self, depth, elem_num):
        #raise RuntimeError
        arity = c_backend.PyTuple_len(self._ptr)
        if not types:
            raise MissingTypeHint
        if arity != len(self.sig.__tuple_params__) and self.call_fn:
            raise TypeError("the type hint for returning tuple of fn `{}` "
                            "and the return tuple value are not of "
                            "the same length".format(self.call_fn._fn_name))
        elif arity != len(self.sig.__tuple_params__):
            raise TypeError("type hint for PyTuple is of wrong length")
        tuple_elems = []
        for i, t in enumerate(self.sig.__tuple_params__):
            pytype = _extract_pytypes(
                self._ptr, call_fn=self.call_fn, depth=depth + 1, elem_num=i,
                downcast=t)
            tuple_elems.append(pytype)
        return tuple(tuple_elems)

    @staticmethod
    def from_tuple(val: tuple):
        raise NotImplementedError

FIND_TYPE = re.compile("type\((.*)\)")
NESTED_TYPE = re.compile("(?P<parent>\w*)<(?P<child>.*)>$")


def _get_signature_types(params):
    def inner_types(t):
        t = t.strip()
        match = re.search(NESTED_TYPE, t)
        mutref, ref = False, False
        if match:
            type_ = match.group('parent')
        else:
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
            raise TypeError('type not supported: {}'.format(type_))
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
                inner = match.group('child')
                inner_t = inner_types(inner)
                return typing.List[inner_t]
            elif equiv == 'dict':
                inner = match.group('child')
                k, v = inner.split(',')
                return typing.Dict[inner_types(k), inner_types(v)]
            elif equiv == 'None':
                return RustType(equiv=None, ref=False, mutref=False)

    params = [x for x in params.split(';') if x != '']
    param_types = []
    for p in params:
        param_types.append(re.search(FIND_TYPE, p).group(1))
        param_types[-1] = inner_types(param_types[-1])
    return param_types


def _get_ptr_to_C_obj(obj, signature=None):
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
        raise NotImplementedError


def _extract_pytypes(ref, call_fn=None, depth=0, elem_num=None,
                     downcast=False, sig=False, curr_elem=None):
    if downcast:
        if issubclass(downcast, bool):
            pybool = c_backend.PyTuple_extractPyBool(ref, elem_num)
            return _extract_pytypes(pybool)
        elif issubclass(downcast, int):
            return c_backend.PyTuple_extractPyInt(ref, elem_num)
        elif issubclass(downcast, float):
            if downcast is Float:
                return c_backend.PyTuple_extractPyFloat(ref, elem_num)
            else:
                return c_backend.PyTuple_extractPyDouble(ref, elem_num)
        elif issubclass(downcast, str):
            pystr = c_backend.PyTuple_extractPyString(ref, elem_num)
            return _extract_pytypes(pystr)
        elif issubclass(downcast, typing.Tuple):
            raise NotImplementedError
            pytuple = c_backend.PyTuple_extractPyTuple(ref, elem_num)
            return _extract_pytypes(
                pytuple, depth=depth + 1, sig=sig, curr_elem=elem_num)

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
        if not sig:
            types = call_fn.restype
        else:
            types = sig
        pyobj = PyTuple(ref, types, call_fn=call_fn)
        val = pyobj.to_tuple(depth, elem_num)
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
    else:
        raise TypeError("return type not supported")


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


def bind_rs_crate_funcs(mod, lib, cargo=False, ismodule=False, prefix=None):
    if not isinstance(mod, str):
        # type checking is necessary as it will be passed to Rust
        raise TypeError('`mod` parameter must be a valid string')
    if not cargo:
        manifest = os.path.join(mod, 'Cargo.toml')
        if not os.path.exists(manifest):
            raise OSError("no Cargo(.toml) manifest found")
        entry_point = get_crate_entry(mod, manifest)
    return RustBinds(entry_point, lib, prefix=prefix)

# ==================== #
#   Support classes    #
# ==================== #


class KrateData(object):

    def __init__(self):
        self.obj = c_backend.krate_data_new()

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

# ==================== #
#   Main generators    #
# ==================== #


class RustBinds(object):

    def __init__(self, entry_point, compiled_lib, prefix=None):
        self._FFI = ctypes.cdll.LoadLibrary(compiled_lib)
        self._krate_data = KrateData()
        p = PyString.from_str(entry_point)
        signal = c_backend.parse_src(p, self._krate_data.obj)
        if signal == 1:
            raise Exception(
                "rustypy: failed to generate Rust bindings, the source "
                "code didn't parse, checkout if your library compiles!")
        if prefix is None:
            prefix = "python_bind_"
        prepared_funcs = {}
        with self._krate_data as krate:
            for e in krate:
                decl = e.to_str()
                if decl == "NO_IDX_ERROR":
                    break
                path, decl = decl.split(prefix)
                name, params = decl.split('::', maxsplit=1)
                name = prefix + name
                params = _get_signature_types(params)
                self.decl_C_args(name, params)
                prepared_funcs[name] = self.FnCall(name, params, self._FFI)
        for name, fn in prepared_funcs.items():
            setattr(self, name, fn)

    class FnCall(object):

        def __init__(self, name, argtypes, lib):
            self._rs_fn = getattr(lib, name)
            self._fn_name = name
            self.__type_hints = {'real_return': argtypes.pop()}
            self.__type_hints['argtypes'] = argtypes

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
                raise TypeError("{}() takes exactly {} "
                                "arguments ({} given)".format(
                                    self._fn_name, n_args, g_args))
            prep_args = []
            for x, a in enumerate(args):
                p = self.argtypes[x]
                if p.ref or p.mutref:
                    ref = _get_ptr_to_C_obj(a)
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
                    raise TypeError("argument #{} type of `{}` passed to "
                                    "function `{}` not supported".format(
                                        x, a, self._fn_name))
            result = self._rs_fn(*prep_args)
            if not return_ref:
                # connversion of result to Python objects
                try:
                    python_result = _extract_pytypes(result, call_fn=self)
                except MissingTypeHint:
                    raise TypeError("must add return type of "
                                    "function `{}`".format(self._fn_name))
                return python_result
            elif get_contents:
                arg_refs = []
                for x, r in enumerate(prep_args):
                    if isinstance(r, POINTER(PyString_RS)):
                        arg_refs.append(f_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(PyTuple_RS)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(PyBool_RS)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(ctypes.c_longlong)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(ctypes.c_float)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
                    elif isinstance(r, POINTER(ctypes.c_double)):
                        arg_refs.append(_extract_pytypes(r, call_fn=self))
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
                restype = self.__type_hints['return']
            except KeyError:
                return
            else:
                return restype

        @restype.setter
        def restype(self, annotation):
            self.__type_hints['return'] = annotation

        @property
        def argtypes(self):
            try:
                restype = self.__type_hints['argtypes']
            except KeyError:
                return
            else:
                return restype

        @argtypes.setter
        def argtypes(self, annotations):
            raise AttributeError("private member, cannot be set directly")

    def decl_C_args(self, name, params):
        restype = None
        argtypes = []
        for x, p in enumerate(params, 1):
            if p.equiv is None:
                add_p = ctypes.c_void_p
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
            if p.mutref or p.ref:
                add_p = POINTER(add_p)
            if x <= (len(params) - 1):
                argtypes.append(add_p)
            else:
                restype = add_p
        fn = getattr(self._FFI, "{}".format(name))
        setattr(fn, "restype", restype)
        if len(argtypes) > 0:
            setattr(fn, "argtypes", tuple(argtypes))

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
            msg = "`new` (constructor) method name not defined for struct `{}`" \
                " in module `{}`"
            msg = msg.format(self.name)
            return msg

    class StructPtr(object):
        _ERR_RESERVED = "cannot use `krate` attr name, is a reserved attribute"

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
