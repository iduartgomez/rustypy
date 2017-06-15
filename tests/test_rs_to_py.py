import unittest
import subprocess
import sys
import os
import typing
import shutil

from importlib import import_module
from rustypy.pywrapper import RustFuncGen
from rustypy.rswrapper import bind_rs_crate_funcs
from rustypy.rswrapper import Float, Double, Tuple
from rustypy.rswrapper.ffi_defs import _load_rust_lib


def setUpModule():
    #_load_rust_lib(recmpl=True)  # uncomment to recompile rust lib
    global _py_test_dir
    _py_test_dir = os.path.abspath(os.path.dirname(__file__))
    global _rs_lib_dir
    _rs_lib_dir = os.path.join(os.path.dirname(_py_test_dir), 'src', 'rslib')
    os.remove(os.path.join(_rs_lib_dir, 'librustypy.so'))
    # load sample lib
    ext = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
    pre = {'win32': ''}.get(sys.platform, 'lib')
    global lib_test_entry
    lib_test_entry = os.path.join(_py_test_dir, 'rs_test_lib')
    global lib_test
    lib_test = os.path.join(lib_test_entry, 'target', 'debug',
                            '{}test_lib{}'.format(pre, ext))
    subprocess.run(['cargo', 'build'], cwd=lib_test_entry)


class GenerateRustToPythonBinds(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        prefixes = ["python_bind_", "other_prefix_"]
        cls.bindings = bind_rs_crate_funcs(lib_test_entry, lib_test, prefixes)

    def test_basics_primitives(self):
        # non ref int
        return_val = self.bindings.python_bind_int(1)
        self.assertIsInstance(return_val, int)
        self.assertEqual(return_val, 2)
        # ref int
        _, refs = self.bindings.python_bind_ref_int(
            1, return_ref=True, get_contents=True)
        self.assertEqual(refs[0], 2)
        # string
        return_val = self.bindings.python_bind_str("From Python.")
        self.assertEqual(return_val, "From Python. Added in Rust.")
        # bool
        return_val = self.bindings.python_bind_bool(True)
        self.assertEqual(return_val, False)

    def test_tuple_conversion(self):
        # tuple
        U = Tuple[int, int]
        self.bindings.python_bind_int_tuple.restype = U
        for i in range(0, 100):
            return_val = self.bindings.python_bind_int_tuple(1, 2)
            self.assertEqual(return_val, (1, 2))

        U = Tuple[str, str]
        self.bindings.python_bind_str_tuple.restype = U
        return_val = self.bindings.python_bind_str_tuple("Some")
        self.assertEqual(return_val, ("Some", "from Rust"))

        # mixed types
        T = Tuple[int, bool, Float, str]
        self.bindings.python_bind_tuple_mixed.restype = T
        return_val = self.bindings.python_bind_tuple_mixed(
            1, True, 2.5, "Some from Rust")
        self.assertEqual(return_val, (1, False, 2.5, "Some from Rust"))

    def test_list_conversion(self):
        # string list
        T = typing.List[str]
        self.bindings.python_bind_list1.add_argtype(0, T)
        self.bindings.python_bind_list1.restype = T
        result = self.bindings.python_bind_list1(["Python", "in", "Rust"])
        self.assertEqual(result, ["Rust", "in", "Python"])

        # list of tuples
        T = typing.List[Tuple[int, Tuple[Float, int]]]
        self.bindings.python_bind_list2.add_argtype(0, T)
        U = typing.List[Tuple[Double, bool]]
        self.bindings.python_bind_list2.restype = U
        result = self.bindings.python_bind_list2(
            [(50, (1.0, 30)), (25, (0.5, 40))])
        self.assertEqual(result, [(0.5, True), (-0.5, False)])

        # list of lists of tuples
        T = typing.List[typing.List[
            Tuple[int, Tuple[Float, int]]]]
        self.bindings.python_bind_nested1_t_n_ls.add_argtype(0, T)
        self.bindings.python_bind_nested1_t_n_ls.restype = T
        result = self.bindings.python_bind_nested1_t_n_ls(
            [[(50, (1.0, 30))], [(25, (0.5, 40))]])
        self.assertEqual(result, [[(50, (1.0, 30))], [(25, (0.5, 40))]])

        # list of tuples of lists
        T = typing.List[Tuple[typing.List[int], Float]]
        self.bindings.python_bind_nested2_t_n_ls.add_argtype(0, T)
        self.bindings.python_bind_nested2_t_n_ls.restype = T
        result = self.bindings.python_bind_nested2_t_n_ls(
            [([1, 2, 3], 0.1), ([3, 2, 1], 0.2)])
        f = []
        for x in result:
            l = []
            for i, y in enumerate(x):
                if isinstance(y, float):
                    e = round(y, 1)
                else:
                    e = y
                l.append(e)
            f.append(tuple(l))
        self.assertEqual(f, [([3, 2, 1], 0.2), ([1, 2, 3], 0.1)])

    def test_dict_conversion(self):
        from rustypy.rswrapper import HashableType
        d = {0: "From", 1: "Python"}
        T = typing.Dict[HashableType('u64'), str]
        R = typing.Dict[HashableType('i64'), str]
        self.bindings.other_prefix_dict.add_argtype(0, T)
        self.bindings.other_prefix_dict.restype = R
        result = self.bindings.other_prefix_dict(d)
        self.assertEqual(result, {0: "Back", 1: "Rust"})

if __name__ == "__main__":
    unittest.main()
