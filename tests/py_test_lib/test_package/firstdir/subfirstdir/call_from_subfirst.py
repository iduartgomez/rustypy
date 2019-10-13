from rustypy.pywrapper import rust_bind

@rust_bind
def subfirst_module() -> None:
    print('... called from first submodule')
