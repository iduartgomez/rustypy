# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python code."""

import os
import sys
import re
import inspect
import typing as typ
from textwrap import dedent
from io import StringIO

from cffi import FFI

def _load_C_API():
    path = os.path.abspath(os.path.dirname(__file__))

    if sys.platform.startswith("win"): ext = "\.dll"
    elif sys.platform == "darwin": ext = "\.dylib"
    else: ext = "\.so"
    libdir = os.path.join(path, 'lib')

    nofile = FileNotFoundError("""\
rustypy hasn't been installed properly: couldn't find the compiled
library file. Uninstall and re-install again using pip.""")
    C_API = None
    if not os.path.exists(libdir): raise nofile
    else:
        rgx = re.compile(r"(rustypy_C_API.*{})".format(ext), re.I)
        for f in os.listdir(libdir):
            if rgx.match(f):
                C_API = FFI()
                C_API.dlopen(os.path.join(libdir, f))
    if not C_API: raise nofile
