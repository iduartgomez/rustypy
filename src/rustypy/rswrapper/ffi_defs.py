"""FFI definitions."""

import os
import sys
import ctypes
import pkg_resources

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
    'usize': 'OpaquePtr',
    'size_t': 'OpaquePtr',
    'void': 'None',
}


class Raw_RS(ctypes.Structure):
    pass


class PyArg_RS(ctypes.Structure):
    pass


class PyString_RS(ctypes.Structure):
    pass


class PyBool_RS(ctypes.Structure):
    pass


class PyTuple_RS(ctypes.Structure):
    pass


class PyList_RS(ctypes.Structure):
    pass


# BEGIN dictionary interfaces
class PyDict_RS(ctypes.Structure):
    pass


class KeyType_RS(ctypes.Structure):
    pass


class DrainPyDict_RS(ctypes.Structure):
    pass
# END dictionary interfaces


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
    c_backend.parse_src.restype = POINTER(PyString_RS)

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
    #c_backend.pydict_get_mut_element.restype = c_void_p
    c_backend.pydict_get_drain.argtypes = (
        POINTER(PyDict_RS), POINTER(KeyType_RS))
    c_backend.pydict_get_drain.restype = POINTER(DrainPyDict_RS)
    c_backend.pydict_drain_element.argtypes = (
        POINTER(DrainPyDict_RS), POINTER(KeyType_RS))
    c_backend.pydict_drain_element.restype = POINTER(PyArg_RS)
    c_backend.pydict_get_kv.argtypes = (ctypes.c_int, POINTER(PyArg_RS))
    c_backend.pydict_get_kv.restype = POINTER(PyArg_RS)
    c_backend.pydict_free_kv.argtypes = (POINTER(PyArg_RS),)

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


def _load_rust_lib(recmpl=False):
    ext = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
    pre = {'win32': ''}.get(sys.platform, 'lib')
    libfile = "{}rustypy{}".format(pre, ext)
    lib = pkg_resources.resource_filename('rslib', libfile)
    if (not os.path.exists(lib)) or recmpl:
        print("   library not found at: {}".format(lib))
        print("   compiling with Cargo")
        import subprocess
        path = os.path.dirname(lib)
        subprocess.run(['cargo', 'build', '--release'], cwd=path)
        #subprocess.run(['cargo', 'build'], cwd=path)
        import shutil
        cp = os.path.join(path, 'target', 'release', libfile)
        #cp = os.path.join(path, 'target', 'debug', libfile)
        if os.path.exists(lib):
            os.remove(lib)
        shutil.copy(cp, path)
        _load_rust_lib()
    else:
        from ..__init__ import __version__ as curr_ver
        # check that is the same version
        lib_ver = curr_ver
        # load the library
        if lib_ver != curr_ver:
            compile_rust_lib(recmpl=True)
        else:
            globals()['c_backend'] = ctypes.cdll.LoadLibrary(lib)
            config_ctypes()


def get_rs_lib():
    if not c_backend:
        _load_rust_lib()
    return c_backend
