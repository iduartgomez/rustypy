import os
import pathlib
import subprocess
import sys
import unittest

from rustypy.pywrapper import RustFuncGen

_test_lib_dir = None


def setUpModule():
    # from rustypy.rswrapper.ffi_defs import _load_rust_lib
    # _load_rust_lib(recmpl=True)  # uncomment to recompile rust lib
    _py_test_dir = os.path.abspath(os.path.dirname(__file__))
    global _test_lib_dir
    _test_lib_dir = pathlib.Path(_py_test_dir, 'py_test_lib', )

    # set python path
    mod_path = _test_lib_dir
    sys.path.append(str(mod_path))
    print("PYTHON PATH: {}\n".format(sys.path))
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

    # def test_basics_primitives(self):
    #     p = subprocess.run(['cargo', 'test', '--test', 'primitives'],
    #                        cwd=_test_lib_dir)
    #     self.assertEqual(p.returncode, 0,
    #                      'failed Rust integration test `basics_primitives`')

    def test_basics_nested_types(self):
        p = subprocess.run(['cargo', 'test', '--test', 'nested_types'],
                           cwd=_test_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_nested_types`')

    def test_nested_modules(self):
        p = subprocess.run(['cargo', 'test', '--test', 'submodules'],
                           cwd=_test_lib_dir)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `nested modules`')


if __name__ == "__main__":
    unittest.main()
