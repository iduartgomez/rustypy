import os
import pathlib
import subprocess
import sys
import unittest

from rustypy.pywrapper import RustFuncGen

_test_lib_dir = None
_rs_lib_path = None


def setUpModule():
    # from rustypy.rswrapper.ffi_defs import _load_rust_lib
    # _load_rust_lib(recmpl=True)  # uncomment to recompile rust lib
    py_test_dir = os.path.abspath(os.path.dirname(__file__))
    global _test_lib_dir
    _test_lib_dir = pathlib.Path(py_test_dir, 'py_test_lib')
    global _rs_lib_path
    _rs_lib_path = pathlib.Path(py_test_dir).parent.joinpath('src', 'librustypy')

    # set python path
    mod_path = _test_lib_dir
    sys.path.append(str(mod_path))
    # set env python path
    original_env = os.getenv('PYTHONPATH')
    if original_env:
        new_env = original_env + os.pathsep + str(mod_path)
    else:
        new_env = str(mod_path)
    os.putenv('PYTHONPATH', new_env)


class GeneratePythonToRustBinds(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        prefixes = ["rust_bind_", "other_prefix_"]
        RustFuncGen(with_path=_test_lib_dir.joinpath("test_package"),
                    prefixes=prefixes)

    def test_basics_primitives(self):
        p = subprocess.run(['cargo', 'test', 'primitives'],
                           cwd=str(_test_lib_dir))
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_primitives`')

    def test_basics_nested_types(self):
        p = subprocess.run(['cargo', 'test', 'nested_types'],
                           cwd=str(_test_lib_dir))
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_nested_types`')

    def test_nested_modules(self):
        p = subprocess.run(['cargo', 'test', 'submodules'],
                           cwd=str(_test_lib_dir))
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `nested modules`')


if __name__ == "__main__":
    unittest.main()
