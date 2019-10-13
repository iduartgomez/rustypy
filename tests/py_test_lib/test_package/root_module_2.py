from rustypy.pywrapper import rust_bind

@rust_bind
def root_module_2() -> None:
    print('... called from root module 2')
