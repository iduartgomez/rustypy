from rustypy import rust_bind

@rust_bind
def first_module() -> None:
    print('... called from first module')
