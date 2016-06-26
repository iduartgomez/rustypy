def bind_python_package_funcs():
    from rustypy.pywrapper import RustFuncGen
    import os
    import inspect
    caller = inspect.stack()[1]
    info = dict(inspect.getmembers(caller.frame))
    path = info["f_globals"]["__file__"]
    path = os.path.abspath(path)
    RustFuncGen(with_path=path)

def _compile_API():
    from cffi import FFI
    ffi = FFI()
    ffi.set_source("rustypy_C_API", """
    """)
    if "tmpdir" not in globals():
        tmpdir = "./lib"
    output = ffi.compile(tmpdir=tmpdir)
    print("Output in: \n{}".format(output))

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
    parser = argparse.ArgumentParser(prog="rustypy",
        description="Generates bindings from/to Rust/Python for the specified "
        + "package or module.")
    parser.add_argument("lang",
        help="target language of the bindings generated, \
    ie. `python` to generate binds to Python functions from Rust",
        choices=['rust', 'python'])
    parser.add_argument("path",
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
        raise NotImplementedError("rustypy: rust bind generator not implemented")
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
