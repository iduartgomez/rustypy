from rustypy.pywrapper import rust_bind


@rust_bind
def first_module() -> None:
    print('... called from first module')


if __name__ == "__main__":
    first_module()
