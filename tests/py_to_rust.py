import unittest
import subprocess
import sys
import os

from rustypy.pywrapper import RustFuncGen


def setUpModule():
    from importlib import import_module
    path = os.path.abspath(os.path.dirname(__file__))
    global _rustypy_test_path
    _rustypy_test_path = path

class GenerateRustToPythonBinds(unittest.TestCase):
    pass


class GeneratePythonToRustBinds(unittest.TestCase):

    def setUp(self):
        self._basics= os.path.join(_rustypy_test_path,
            'test_package', 'basics', 'rustypy_pybind.rs')
        self._mod = os.path.join(_rustypy_test_path,
            'test_package', 'rustypy_pybind.rs')
        f = open(self._basics, 'w')
        f.close()
        f = open(self._mod, 'w')
        f.close()

    def test_basics_primitives(self):
        self.set_python_path('test_package', 'basics')
        import test_package.basics.primitives as test
        self.gen = RustFuncGen(module=test)

        p = subprocess.run('cargo test --test primitives', shell=True)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_primitives`')

    def test_basics_nested_types(self):
        self.set_python_path('test_package', 'basics')
        import test_package.basics.nested_types as test
        self.gen = RustFuncGen(module=test)

        p = subprocess.run('cargo test --test nested_types', shell=True)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `basics_nested_types`')

    def test_nested_modules(self):
        self.set_python_path(_rustypy_test_path)
        init_package = os.path.join(
            _rustypy_test_path, 'test_package', '__init__.py')
        subprocess.run('python {}'.format(init_package), shell=True)

        p = subprocess.run('cargo test --test submodules', shell=True)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `nested modules`')

    def set_python_path(self, *path):
        mod_path = os.path.join(_rustypy_test_path, *path)
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
        #delete files
        os.remove(self._basics)
        os.remove(self._mod)

if __name__ == "__main__":
    unittest.main()
