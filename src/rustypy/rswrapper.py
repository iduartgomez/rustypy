# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python."""

import os
import sys
import pkg_resources
import re
import types
import typing

from string import Template
from cffi import FFI

ffi = FFI()
ffi.cdef("""
    typedef struct KrateData *KrateData;
    typedef struct {
        char* ptr;
        unsigned long len;
    } PyString;

    KrateData* krate_data_new();
    void krate_data_free(KrateData *ptr);
    long krate_data_len(KrateData *ptr);
    PyString krate_data_iter(KrateData *ptr, uint32_t idx);

    int parse_src(char *mod_, KrateData *krate_data);
""")
RS_TYPE_CONVERSION = {
    'c_char': 'str',
    'c_double': 'float',
    'c_float': 'float',
    'c_int': 'int',
    'c_long': 'int',
    'c_longlong': 'int',
    'c_schar': 'str',
    'c_short': 'int',
    'c_uint': 'int',
    'c_ulong': 'int',
    'c_ulonglong': 'int',
    'c_ushort': 'int',
    'size_t': 'int',
    'ssize_t': 'int',
    'u64': 'int',
    'u32': 'int',
    'u16': 'int',
    'u8': 'int',
    'i64': 'int',
    'i32': 'int',
    'i16': 'int',
    'i8': 'int',
    'f32': 'float',
    'f64': 'float',
    'bool': 'bool',
    'Vec': 'list',
    'HashMap': 'dict',
    'tuple': 'tuple',
    'void': 'None'
}
FIND_TYPE = re.compile("type\((.*)\)")
NESTED_TYPE = re.compile("(?P<parent>\w*)<(?P<child>.*)>$|\((?P<tuple>.*)\)$")

global rslib
rslib = None


def load_rust_lib(recmpl=False):
    if sys.platform.startswith("win"):
        ext = ".dll"
    elif sys.platform == "darwin":
        ext = ".dylib"
    else:
        ext = ".so"
    lib = pkg_resources.resource_filename('rslib', "librustypy{}".format(ext))
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
            globals()['rslib'] = ffi.dlopen(lib)

if not rslib:
    load_rust_lib()

# ==================== #
#   Helper Functions   #
# ==================== #

from collections import namedtuple
RustType = namedtuple('RustType', ['equiv', 'ref', 'mutref'])


def _get_signature_types(params):
    def inner_types(t):
        t = t.strip()
        match = re.search(NESTED_TYPE, t)
        mutref, ref = False, False
        if match:
            if match.group('parent'):
                type_ = match.group('parent')
            else:
                type_ = 'tuple'
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
                return float
            elif equiv == 'str':
                return RustType(equiv=str, ref=True, mutref=mutref)
            elif equiv == 'None':
                return RustType(equiv=None, ref=ref, mutref=mutref)
            elif equiv == 'bool':
                return bool
            elif equiv == 'list':
                inner = match.group('child')
                inner_t = inner_types(inner)
                return typing.List[inner_t]
            elif equiv == 'dict':
                inner = match.group('child')
                k, v = inner.split(',')
                return typing.Dict[inner_types(k), inner_types(v)]
            elif equiv == 'tuple':
                inner = match.group('tuple')

    params = [x for x in params.split(';') if x != '']
    param_types = []
    for p in params:
        param_types.append(re.search(FIND_TYPE, p).group(1))
        param_types[-1] = inner_types(param_types[-1])
    return param_types


def get_memref_to_obj(obj):
    if isinstance(obj, int):
        return ffi.new('long *', obj)
    if isinstance(obj, str):
        return ffi.new("char[]", obj.encode())


def _get_crate_entry(mod, manifest):
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
        entry_point = _get_crate_entry(mod, manifest)
    return RustBinds(entry_point, lib, prefix=prefix)

# ==================== #
#   Support classes    #
# ==================== #


class KrateData(object):

    def __init__(self):
        self.obj = rslib.krate_data_new()
        self._idx = 0

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        rslib.krate_data_free(self.obj)

    def __iter__(self):
        self._len = rslib.krate_data_len(self.obj)
        return self

    def __next__(self):
        if (self._len - 1) == -1 or self._idx > (self._len - 1):
            self._idx = 0
            raise StopIteration
        val = rslib.krate_data_iter(self.obj, self._idx)
        self._idx += 1
        return val


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


class RsFunc(object):

    def __init__(self):
        pass

# ==================== #
#   Main generators    #
# ==================== #


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


class RustBinds(object):

    def __init__(self, entry_point, compiled_lib, prefix=None):
        self._FFI = FFI()
        self.lib = self._FFI.dlopen(compiled_lib)
        self._krate_data = KrateData()
        rslib.parse_src(entry_point.encode(), self._krate_data.obj)
        if prefix is None:
            prefix = "python_bind_"
        prepared_funcs = {}
        with self._krate_data as krate:
            for e in krate:
                decl = ffi.unpack(e.ptr, e.len).decode("utf-8")
                path, decl = decl.split(prefix)
                name, params = decl.split('::', maxsplit=1)
                name = prefix + name
                params = _get_signature_types(params)
                self.decl_C_def(name, params)
                prepared_funcs[name] = self.FnCall(self.lib, name, params)
        for name, fn in prepared_funcs.items():
            setattr(self, name, fn)

    def decl_C_def(self, name, params):
        print(name, params)
        params_str = ""
        for x, p in enumerate(params, 1):
            add_p = ""
            if p.equiv is None:
                add_p = "void"
            elif issubclass(p.equiv, int):
                if not p.mutref and not p.ref:
                    add_p = "long"
                else:
                    add_p = "long *"
            elif issubclass(p.equiv, str):
                add_p = "char *"
            if x < (len(params) - 2) and x != len(params):
                params_str += (add_p + ',')
            elif x == (len(params) - 1):
                params_str += add_p
            else:
                return_str = add_p
        cdef = """
            {return_type} {name}({params});
        """.format(
            return_type=return_str,
            name=name,
            params=params_str)
        print(cdef)
        self._FFI.cdef(cdef)

    class FnCall(object):

        def __init__(self, lib, name, params, type_checking=False):
            self._rs_fn = getattr(lib, name)
            self._fn_name = name
            self._return_type = params.pop()
            self._params = params
            self._type_checking = type_checking

        def __call__(self, *args):
            n_args = len(self._params)
            g_args = len(args)
            if g_args != n_args:
                raise TypeError("{}() takes exactly {} "
                                "arguments ({} given)".format(
                                    self._fn_name, n_args, g_args))
            prep_args = []
            for x, a in enumerate(args):
                p = self._params[x]
                if p.ref or p.mutref:
                    ref = get_memref_to_obj(a)
                    prep_args.append(ref)
                else:
                    prep_args.append(a)
            if self._type_checking:
                raise NotImplemented("optional type checking not implemented")
            result = self._rs_fn(*prep_args)
            arg_refs = []
            for x, r in enumerate(prep_args):
                if isinstance(r, ffi.CData):
                    if ffi.typeof(r) is ffi.typeof("char[]"):
                        arg_refs.append(ffi.string(r))
                    else:
                        arg_refs.append(r[0])
                else:
                    arg_refs.append(r)
            # connversion of result to Python objects
            if isinstance(result, ffi.CData):
                if ffi.typeof(result) is ffi.typeof("long *"):
                    result = result[0]
                if ffi.typeof(result) is ffi.typeof("char *"):
                    result = ffi.string(result).decode()
            return result, arg_refs
