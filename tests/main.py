import unittest
import subprocess
import sys
import os
import typing
from importlib import import_module

from rustypy.pywrapper import RustFuncGen
from rustypy.rswrapper import load_rust_lib, Float, Double
from rustypy.rswrapper import bind_rs_crate_funcs


def setUpModule():
    global _py_test_dir
    _py_test_dir = os.path.abspath(os.path.dirname(__file__))
    global _rs_lib_dir
    _rs_lib_dir = os.path.join(os.path.dirname(_py_test_dir), 'src', 'rslib')
    # compile rust lib
    load_rust_lib(recmpl=True)


@unittest.skip
class GenerateRustToPythonBinds(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if sys.platform.startswith("win"):
            ext = ".dll"
        elif sys.platform == "darwin":
            ext = ".dylib"
        else:
            ext = ".so"
        source = os.path.join(_py_test_dir, 'rs_test_lib')
        lib_test = os.path.join(source, 'target', 'debug', 'libtest_lib' + ext)
        cls.bindings = bind_rs_crate_funcs(source, lib_test)

    #@unittest.skip
    def test_basics_primitives(self):
        # non ref
        return_val = self.bindings.python_bind_int(1)
        self.assertIsInstance(return_val, int)
        self.assertEqual(return_val, 2)
        # ref
        _, refs = self.bindings.python_bind_ref_int(1, return_ref=True)
        self.assertEqual(refs[0], 2)
        # string
        return_val = self.bindings.python_bind_str("From Python.")
        self.assertEqual(return_val, "From Python. Added in Rust.")
        return_val = self.bindings.python_bind_str_by_ref("From Python.")
        self.assertEqual(return_val, "From Python. Added in Rust.")

    def test_tuple_conversion(self):
        # tuple
        U = typing.Tuple[int, int]
        self.bindings.python_bind_tuple.add_return_type(U)
        return_val = self.bindings.python_bind_tuple(1, 2)
        self.assertEqual(return_val, (1, 2))

        # mixed types
        T = typing.Tuple[int, bool, Float, str]
        self.bindings.python_bind_tuple_mixed.add_return_type(T)
        return_val = self.bindings.python_bind_tuple_mixed(
            1, False, 2.5, "Some")
        self.assertEqual(return_val, (1, False, 2.5, "Some"))


#@unittest.skip
class GeneratePythonToRustBinds(unittest.TestCase):

    def setUp(self):
        self._basics = os.path.join(_rs_lib_dir, 'tests',
                                    'test_package', 'basics', 'rustypy_pybind.rs')
        self._mod = os.path.join(_rs_lib_dir, 'tests',
                                 'test_package', 'rustypy_pybind.rs')
        f = open(self._basics, 'w')
        f.close()
        f = open(self._mod, 'w')
        f.close()

    @unittest.skip
    def test_basics_primitives(self):
        self.set_python_path('test_package', 'basics')
        import primitives as test
        self.gen = RustFuncGen(module=test)
        p = subprocess.run(['cargo', 'test', '--test', 'primitives'],
                           cwd=_rs_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_primitives`')

    def test_basics_nested_types(self):
        self.set_python_path('test_package', 'basics')
        import nested_types as test
        self.gen = RustFuncGen(module=test)
        p = subprocess.run(['cargo', 'test', '--test', 'nested_types'],
                           cwd=_rs_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_nested_types`')

    @unittest.skip
    def test_nested_modules(self):
        self.set_python_path()
        init_package = os.path.join(_rs_lib_dir, 'tests',
                                    'test_package', '__init__.py')
        subprocess.run('python {}'.format(init_package), shell=True)
        p = subprocess.run(['cargo', 'test', '--test', 'submodules'],
                           cwd=_rs_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `nested modules`')

    def set_python_path(self, *path):
        mod_path = os.path.join(_rs_lib_dir, 'tests', *path)
        self._original_path = sys.path.copy()
        sys.path.append(mod_path)

        self._original_env = os.getenv('PYTHONPATH', default="")
        if not self._original_env:
            new_env = mod_path
        else:
            new_env = self._original_env + os.pathsep + mod_path
        os.putenv('PYTHONPATH', new_env)

    def tearDown(self):
        if hasattr(self, '_original_path'):
            sys.path = self._original_path
        if hasattr(self, '_original_env'):
            os.putenv('PYTHONPATH', self._original_env)
        # delete files
        # os.remove(self._basics)
        # os.remove(self._mod)

if __name__ == "__main__":
    unittest.main()
