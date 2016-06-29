def cli():
    import argparse
    import os
    import pip
    from importlib import import_module
    from rustypy.pywrapper import RustFuncGen
    # error messages
    _ext_err = "rustypy: error: target language and file extension " + \
        "are not coherent"
    _pckg_err = "rustypy: error: provide the name of the package " + \
        "or the path to the package"
    _not_found_err = "rustypy: error: package is not installed, you can " + \
        "install packages in development mode using `pip install -e <path>`"
    # CLI
    parser = argparse.ArgumentParser(
        prog="rustypy",
        description="Generates bindings from/to Rust/Python for the specified "
        + "package or module.")
    parser.add_argument(
        "lang",
        help="target language of the bindings generated, \
        ie. `python` to generate binds to Python functions from Rust",
        choices=['rust', 'python'])
    parser.add_argument(
        "path",
        help="absolute path or name of the package/module (must be available \
        from the Python or Cargo, in case of Rust, path")
    group1 = parser.add_mutually_exclusive_group(required=True)
    group1.add_argument("-m", "--module", action="store_true",
                        help="it's a single module, default is a package")
    group1.add_argument("-p", "--package", action="store_true",
                        help="it's a a package (default option)")
    args = parser.parse_args()
    if args.module:
        ismodule = True
    else:
        ismodule = False
    lang = args.lang
    path = args.path
    prefix = None
    # run script
    if os.sep in path and not os.path.exists(path):
        SystemExit("rustypy: error: the path does not exist")
    if lang == 'rust':
        raise NotImplementedError(
            "rustypy: rust bind generator not implemented")
    else:
        ext = ".py"
    pckg, module = False, False
    if ismodule:
        if "." not in path:
            module, is_path = path, False
        if path[-3:] != ext:
            raise SystemExit(_ext_err)
        else:
            module, is_path = True, True
    else:
        if "." in path:
            raise SystemExit(_pckg_err)
        elif os.sep in path:
            pckg, is_path = True, True
        else:
            pckg, is_path = path, False

    if lang == 'python':
        if is_path and pckg:
            RustFuncGen(with_path=path, prefix=prefix)
        elif is_path and module:
            RustFuncGen(with_path=path, module=True, prefix=prefix)
        elif pckg:
            location = None
            for x in pip.get_installed_distributions(local_only=True):
                if x._key == pckg:
                    location = x.location
                    break
            if not location:
                raise SystemExit(_not_found_err)
            RustFuncGen(with_path=location, prefix=prefix)
        elif module:
            mod = import_module(module)
            RustFuncGen(module=mod, prefix=prefix)
    if ismodule:
        print("rustypy: binds for module `{}` generated".format(path))
    else:
        print("rustypy: binds for package `{}` generated".format(path))


def load_rust_lib(recmpl=False):
    if '_rustypy_rs_lib' in globals().keys() or not recmpl:
        return

    import os
    import sys
    import pkg_resources

    if sys.platform.startswith("win"):
        ext = ".dll"
    elif sys.platform == "darwin":
        ext = ".dylib"
    else:
        ext = ".so"
    lib = pkg_resources.resource_filename(
        'rslib', "librustypy{}".format(ext))
    if not os.path.exists(lib) or recmpl:
        print("   compiled library not found at: {}".format(lib))
        print("   compiling with Cargo")
        import subprocess
        path = os.path.dirname(lib)
        subprocess.run(['cargo', 'build', '--release'], cwd=path)
        import shutil
        cp = os.path.join(path, 'target', 'release', "librustypy{}".format(ext))
        shutil.copy(cp, path)
    else:
        print("   library already exists")
        from .__init__ import __version__ as curr_ver
        # check that is the same version
        lib_ver = curr_ver
        # load the library
        if lib_ver != curr_ver:
            compile_rust_lib(recompile=True)
        else:
            global _rustypy_rs_lib
            from cffi import FFI
            ffi = FFI()
            _rustypy_rs_lib = ffi.dlopen(lib)
            return _rustypy_rs_lib

def get_version():
    import pkg_resources
    try:
        rustypy_ver = pkg_resources.require("rustypy")[0].version
    except ImportError:
        import os
        import re
        p = os.path.join(os.path.dirname(__file__), '__init__.py')
        rustypy_ver = re.compile(r"^__version__ = '(.*)'")
        with open(p) as f:
            for l in f:
                ver = re.match(rustypy_ver, l)
                if ver:
                    rustypy_ver = ver.group(1)
                    break
    return rustypy_ver
