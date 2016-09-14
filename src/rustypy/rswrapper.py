# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python."""

import os
import sys
import pkg_resources
import re

from cffi import FFI

ffi = FFI()
ffi.cdef("""
    typedef struct KrateData *KrateData;
    typedef struct {
        char* ptr;
        int len;
    } PyString;

    KrateData* krate_data_new();
    void krate_data_free(KrateData *ptr);
    long krate_data_len(KrateData *ptr);
    PyString krate_data_iter(KrateData *ptr, uint32_t idx);

    int parse_src(char *mod_, KrateData *krate_data);
""")
# long krate_data_len(KrateData *ptr);


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


def get_signature_types(types):
    pass


class RsTypes(object):
    # std_vec
    # std_hashmap
    c_long = "int"
    c_double = "float"
    c_char = "char"


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
        types = get_signature_types(method)
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

    def add_child_mod(self, mod):
        setattr(self, mod.name, mod)

    def add_child_func(self, func):
        setattr(self, func.name, struct)

    def add_child_struct(self, struct):
        setattr(self, struct.name, struct)


class RustBinds(object):

    def __init__(self, entry_point, prefix=None):
        self._krate = KrateData()
        rslib.parse_src(entry_point.encode(), self._krate.obj)
        with self._krate as krate:
            for e in krate:
                s = ffi.unpack(e.ptr, e.len).decode("utf-8")
                print(s)

# ==================== #
#   Helper Functions   #
# ==================== #


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


def bind_rs_crate_funcs(mod, cargo=False, ismodule=False):
    if not isinstance(mod, str):
        # type checking is necessary as it will be passed to Rust
        raise TypeError('`mod` parameter must be a valid string')
    if not cargo:
        manifest = os.path.join(mod, 'Cargo.toml')
        if not os.path.exists(manifest):
            raise OSError("no Cargo(.toml) manifest found")
        entry_point = _get_crate_entry(mod, manifest)
    return RustBinds(entry_point)
