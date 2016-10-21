import unittest
import subprocess
import sys
import os
import typing
import shutil

from importlib import import_module
from rustypy.pywrapper import RustFuncGen
from rustypy.rswrapper import bind_rs_crate_funcs, load_rust_lib
from rustypy.rswrapper import Float, Double


def setUpModule():
    # compile rust lib
    load_rust_lib(recmpl=False)
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


#@unittest.skip
class GenerateRustToPythonBinds(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.bindings = bind_rs_crate_funcs(lib_test_entry, lib_test)

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
        U = typing.Tuple[int, int]
        self.bindings.python_bind_int_tuple.restype = U
        for i in range(0, 100):
            return_val = self.bindings.python_bind_int_tuple(1, 2)
            self.assertEqual(return_val, (1, 2))

        U = typing.Tuple[str, str]
        self.bindings.python_bind_str_tuple.restype = U
        return_val = self.bindings.python_bind_str_tuple("Some")
        self.assertEqual(return_val, ("Some", "from Rust"))

        # mixed types
        T = typing.Tuple[int, bool, Float, str]
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
        T = typing.List[typing.Tuple[int, typing.Tuple[Float, int]]]
        self.bindings.python_bind_list2.add_argtype(0, T)
        U = typing.List[typing.Tuple[Double, bool]]
        self.bindings.python_bind_list2.restype = U
        result = self.bindings.python_bind_list2(
            [(50, (1.0, 30)), (25, (0.5, 40))])
        self.assertEqual(result, [(0.5, True), (-0.5, False)])

        # list of lists of tuples
        T = typing.List[typing.List[
            typing.Tuple[int, typing.Tuple[Float, int]]]]
        self.bindings.python_bind_nested1_t_n_ls.add_argtype(0, T)
        self.bindings.python_bind_nested1_t_n_ls.restype = T
        result = self.bindings.python_bind_nested1_t_n_ls(
            [[(50, (1.0, 30))], [(25, (0.5, 40))]])
        self.assertEqual(result, [[(50, (1.0, 30))], [(25, (0.5, 40))]])

        # list of tuples of lists
        T = typing.List[typing.Tuple[typing.List[int], Float]]
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
        from rustypy import HashableType
        from rustypy.rswrapper import PyDict, UnsignedLongLong
        d = {0: "From", 1: "Python"}
        T = typing.Dict[HashableType('u64'), str]
        R = typing.Dict[HashableType('i64'), str]
        self.bindings.python_bind_dict.add_argtype(0, T)
        self.bindings.python_bind_dict.restype = R
        result = self.bindings.python_bind_dict(d)
        self.assertEqual(result, {0: "Back", 1: "Rust"})


@unittest.skip
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
        self.gen = RustFuncGen(module=test)
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


@unittest.skip
class ExtractionFailures(unittest.TestCase):

    def setUp(self):
        f = open(self._basics, 'w')
        f.close()
        f = open(self._mod, 'w')
        f.close()

    @classmethod
    def setUpClass(cls):
        cls._basics = os.path.join(_rs_lib_dir, 'tests',
                                   'test_package', 'basics', 'rustypy_pybind.rs')
        cls._mod = os.path.join(_rs_lib_dir, 'tests',
                                'test_package', 'rustypy_pybind.rs')

    def test_failure(self):
        set_python_path(self, 'test_package', 'basics')
        import nested_types as test
        self.gen = RustFuncGen(module=test)
        p = subprocess.run(['cargo', 'test', '--test', 'common/nested_types',
                            '--', '--nocapture'],
                           cwd=_rs_lib_dir, universal_newlines=True,
                           stderr=subprocess.STDOUT)
        print("stdout:\n", p.stdout)
        self.assertEqual(p.returncode, 0,
                         'failed Rust integration test `nested types`')

    def tearDown(self):
        if hasattr(self, '_original_path'):
            sys.path = self._original_path
        if hasattr(self, '_original_env'):
            os.putenv('PYTHONPATH', self._original_env)
        f = open(self._basics, 'w')
        f.close()
        f = open(self._mod, 'w')
        f.close()


if __name__ == "__main__":
    unittest.main()
