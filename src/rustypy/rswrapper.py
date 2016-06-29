# -*- coding: utf-8 -*-
"""Generates code for calling Rust from Python code."""

import os
import sys
import re
import inspect
import typing as typ
from textwrap import dedent
from io import StringIO
import pkg_resources

try:
    rslib = globals()['_rustypy_rs_lib']
except KeyError:
    from .scripts import load_rust_lib
    rslib = load_rust_lib()

# ==================== #
#   Helper Functions   #
# ==================== #

def bind_rs_pckg_funcs():
    pass
