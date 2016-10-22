if __name__ == '__main__':
    from rustypy.pywrapper import bind_py_pckg_funcs
    prefixes = ["rust_bind_", "other_prefix_"]
    bind_py_pckg_funcs(prefixes=prefixes)
