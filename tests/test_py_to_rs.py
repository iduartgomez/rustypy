import unittest
import subprocess
import sys
import os
import typing
import shutil

from importlib import import_module
from rustypy.pywrapper import RustFuncGen
from rustypy.rswrapper import bind_rs_crate_funcs
from rustypy.rswrapper import Float, Double
from rustypy.rswrapper.ffi_defs import _load_rust_lib


def setUpModule():
    #_load_rust_lib(recmpl=True)  # uncomment to recompile rust lib
    global _py_test_dir
    _py_test_dir = os.path.abspath(os.path.dirname(__file__))
    global _rs_lib_dir
    _rs_lib_dir = os.path.join(os.path.dirname(_py_test_dir), 'src', 'rslib')
    # load sample lib
    ext = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
    pre = {'win32': ''}.get(sys.platform, 'lib')
    global lib_test_entry
    lib_test_entry = os.path.join(_py_test_dir, 'rs_test_lib')
    global lib_test
    lib_test = os.path.join(lib_test_entry, 'target', 'debug',
                            '{}test_lib{}'.format(pre, ext))
    subprocess.run(['cargo', 'build'], cwd=lib_test_entry)


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


class GeneratePythonToRustBinds(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls._basics = os.path.join(_rs_lib_dir, 'tests',
                                   'test_package', 'basics', 'rustypy_pybind.rs')
        cls._mod = os.path.join(_rs_lib_dir, 'tests',
                                'test_package', 'rustypy_pybind.rs')

    def setUp(self):
        f = open(self._basics, 'w')
        f.close()
        f = open(self._mod, 'w')
        f.close()

    def test_basics_primitives(self):
        set_python_path(self, 'test_package', 'basics')
        import primitives as test
        prefixes = ["rust_bind_", "other_prefix_"]
        self.gen = RustFuncGen(module=test, prefixes=prefixes)
        #
        src = os.path.join(_rs_lib_dir, 'tests', 'python', 'primitives.rs')
        dst = os.path.join(_rs_lib_dir, 'tests')
        shutil.copy(src, dst)
        self._copied = os.path.join(dst, 'primitives.rs')
        #
        p = subprocess.run(['cargo', 'test', '--test', 'primitives'],
                           cwd=_rs_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_primitives`')

    def test_basics_nested_types(self):
        set_python_path(self, 'test_package', 'basics')
        import nested_types as test
        self.gen = RustFuncGen(module=test)
        #
        src = os.path.join(_rs_lib_dir, 'tests', 'python', 'nested_types.rs')
        dst = os.path.join(_rs_lib_dir, 'tests')
        shutil.copy(src, dst)
        self._copied = os.path.join(dst, 'nested_types.rs')
        #
        p = subprocess.run(['cargo', 'test', '--test', 'nested_types'],
                           cwd=_rs_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_nested_types`')

    def test_nested_modules(self):
        set_python_path(self)
        init_package = os.path.join(_rs_lib_dir, 'tests',
                                    'test_package', '__init__.py')
        subprocess.run('python {}'.format(init_package), shell=True)
        #
        src = os.path.join(_rs_lib_dir, 'tests', 'python', 'submodules.rs')
        dst = os.path.join(_rs_lib_dir, 'tests')
        shutil.copy(src, dst)
        self._copied = os.path.join(dst, 'submodules.rs')
        #
        p = subprocess.run(['cargo', 'test', '--test', 'submodules'],
                           cwd=_rs_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `nested modules`')

    def tearDown(self):
        if hasattr(self, '_original_path'):
            sys.path = self._original_path
        if hasattr(self, '_original_env'):
            os.putenv('PYTHONPATH', self._original_env)
        os.remove(self._copied)

if __name__ == "__main__":
    unittest.main()
