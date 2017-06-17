# -*- coding: utf-8 -*-
"""Generates code for calling Python from Rust code."""

import collections.abc as abc
import inspect
import os
import random
import sys
import typing
from collections import namedtuple
from importlib import import_module, invalidate_caches
from io import StringIO
from string import Template, ascii_letters
from textwrap import dedent, indent
from types import FunctionType

from .rswrapper.rswrapper import Double, Float, Tuple
from .scripts import get_version

rustypy_ver = get_version()

tab = "    "  # for formatting output
CONTAINERS = (
    "PyList", "PyDict", "PySet", "PyTuple", "Vec", "HashMap", "Set", "tuple",
)
PRIMITIVES = (
    "String", "c_long", "c_double", "bool",
)


class ModuleStruct(object):
    _function = Template("""
        pub fn $name(&self,
$parameters
        ) $return_type {
            $body
        }
    """)
    _fn_body = Template("""
            let func = self.module.get(*(self.py), "$name").unwrap();
$convert
            let result = func.call(*(self.py), $args None).unwrap();
$extract_values
            return $return_value
    """)
    _fn_arg = "let {0} = {0}.to_py_object(*(self.py));"

    def __init__(self, module, objs, pckg_tree=None):
        self.name = module.split('.')
        if pckg_tree:
            pckg_tree.walk(self.name[1:], self)
        self._extractors = {}
        self._var_names = set()
        code = StringIO()
        for obj in objs:
            if isinstance(obj, RustFuncGen.Func):
                self._extractors[obj.name] = extractors = []
                params, pyconstructors, argnames = [], [], []
                pn = [x[0] for x in obj.parameters]
                for x, param in enumerate(obj.parameters):
                    argnames.append(param[0] + ', ')
                    pyarg_constructor = self._fn_arg.format(param[0])
                    p, _ = self.unwrap_types(param[1], "", "")
                    if p[0] != '(':
                        p = p[1:-1]
                    sig = param[0] + ': ' + p
                    if x + 1 != len(obj.parameters):
                        sig += ",\n"
                        pyarg_constructor += "\n"
                    params.append(indent(sig, tab * 3))
                    pyconstructors.append(indent(pyarg_constructor, tab * 3))
                signature, return_val = self.unwrap_types(
                    obj.rsreturn, "", "",
                    extractors=extractors,
                    arg_names=pn,
                    init=True,
                    parent="result")
                if signature[0] == '(':
                    return_type = "-> " + signature
                elif len(obj.rsreturn) == 1 \
                        and obj.rsreturn[0] == 'PyObject::None':
                    return_type, return_val = "", ""
                else:
                    return_type = "-> " + signature[1:-1]
                if len(argnames) > 0:
                    args = "(" + "".join(argnames) + "),"
                else:
                    args = "NoArgs,"
                body = self._fn_body.substitute(
                    name=obj.name,
                    convert="".join(pyconstructors),
                    args=args,
                    extract_values="".join(extractors),
                    return_value=return_val,
                )
                code.write(self._function.substitute(
                    name=obj.name,
                    parameters="".join(params),
                    return_type=return_type,
                    body=body,
                ))
        self._generated_functions = code

    _extract_tuple_field = """let {var_name} = {parent}.get_item
(*(self.py),\
{position}).unwrap().extract::<{type_}>(*(self.py)).unwrap();\n"""
    _extract_tuple = """let {var_name} = {parent}.get_item(\
*(self.py), {position}).unwrap();\n"""
    _extract_type = """let {var_name} = {parent}.extract::<{type_}>(*(self.py)).\
unwrap();\n"""
    _iter_list = """let mut {var_name} = vec![];
for e in {parent}.iter(*(self.py)).unwrap() {{
"""
    _collect_from_list = """{var_name}.push(e.unwrap().extract::<{type_}>(*(self.py))\
.unwrap());\n"""
    _push_tuple_into_vec = """let {var_name} = ({subtypes});
{parent}.push({var_name});
"""
    _iter_dict = """let mut {var_name} = HashMap::new();
let {var_name}_d: PyDict = PyDict::downcast_from(*(self.py), {parent}).unwrap();
for (key, value) in {var_name}_d.items(*(self.py)) {{
"""

    def unwrap_types(self,
                     iterable, sig, return_val,
                     extractors=None, arg_names=None,
                     in_tuple=False, in_list=False, in_dict=False,
                     init=False, parent=None, depth=1):
        append_to_sig, append_to_val = [], []
        for i, type_ in enumerate(iterable):
            if i > 0:
                subtype, subreturnval = ", ", ", "
            else:
                subtype, subreturnval = "", ""
            if (not init or not isinstance(type_, tuple)) \
                    and extractors is not None:
                var_name = rnd_var_name(self, arg_names)
                if in_tuple and not isinstance(type_, tuple):
                    xtc = self._extract_tuple_field.format(
                        var_name=var_name,
                        parent=parent,
                        position=i,
                        type_=type_,)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif in_tuple and type_[0] == 'tuple':
                    xtc = self._extract_tuple.format(
                        var_name=var_name,
                        parent=parent,
                        position=i,)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif in_list and not isinstance(type_, tuple):
                    xtc = self._collect_from_list.format(
                        var_name=in_list,
                        type_=type_,)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif in_list and type_[0] == 'tuple':
                    xtc = "let {var_name} = e.unwrap();\n".format(
                        var_name=var_name)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif in_dict and not isinstance(type_, tuple):
                    if i == 0:
                        xtc = "let dict_key = " + \
                              "key.extract::<{key_type}>(*(self.py)).unwrap();\n"
                        xtc = xtc.format(key_type=type_)
                    elif i == 1:
                        xtc = "let dict_value = " + \
                              "value.extract::<{value_type}>(*(self.py)).unwrap();\n"
                        xtc = xtc.format(value_type=type_)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif in_dict and type_[0] == 'tuple':
                    if i == 0:
                        xtc = "let {var_name} = key;\n"
                    elif i == 1:
                        xtc = "let {var_name} = value;\n"
                    xtc = xtc.format(var_name=var_name)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif not isinstance(type_, tuple):
                    if type_ == 'PyObject::None':
                        chk_type = 'PyObject'
                    else:
                        chk_type = type_
                    xtc = self._extract_type.format(
                        var_name=var_name,
                        parent=parent,
                        type_=chk_type,)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
                elif type_[0] == 'tuple':
                    xtc = self._extract_tuple.format(
                        var_name=var_name,
                        parent=parent,
                        position=i,)
                    extractors.append(indent(xtc, tab * 2 + tab * depth))
            else:
                var_name = parent

            if isinstance(type_, tuple):
                # is a container type
                if extractors is None:
                    var_name = None
                if type_[0] == 'tuple':
                    subtypes, subreturnvalues = self.unwrap_types(
                        type_[1], "", "",
                        in_tuple=True,
                        extractors=extractors,
                        arg_names=arg_names,
                        parent=var_name,
                        depth=depth + 1)
                    subtype += '(' + subtypes + ')'
                    if extractors is not None:
                        subreturnval += '(' + subreturnvalues + ')'
                        if in_list:
                            xtc = self._push_tuple_into_vec.format(
                                var_name=var_name,
                                subtypes=subreturnvalues,
                                parent=in_list,)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                        if in_dict:
                            if i == 0:
                                xtc = "let dict_key = ({subtypes});\n" \
                                    .format(subtypes=subreturnvalues)
                            elif i == 1:
                                xtc = "let dict_value = ({subtypes});\n" \
                                    .format(subtypes=subreturnvalues)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                elif type_[0] == 'Vec':
                    if extractors is not None:
                        name = rnd_var_name(self, arg_names)
                        if in_tuple:
                            tpl_xtc = rnd_var_name(self, arg_names)
                            xtc = self._extract_tuple.format(
                                parent=parent,
                                position=i,
                                var_name=tpl_xtc)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                            ls_parent = tpl_xtc
                        elif in_dict:
                            if i == 0:
                                ls_parent = 'key'
                            elif i == 1:
                                ls_parent = 'value'
                        elif in_list:
                            ls_parent = 'e'
                        else:
                            ls_parent = parent
                        xtc = self._iter_list.format(
                            parent=ls_parent,
                            var_name=name)
                        extractors.append(indent(xtc, tab * 2 + tab * depth))
                        new_vec = name
                    else:
                        new_vec = True
                    subtypes, subreturnvalues = self.unwrap_types(
                        type_[1], "", "",
                        in_list=new_vec,
                        extractors=extractors,
                        arg_names=arg_names,
                        parent=var_name,
                        depth=depth + 1)
                    var_name = new_vec
                    if in_tuple or in_list or in_dict:
                        subtype += 'Vec<' + subtypes + '>'
                    else:
                        subtype += '<Vec<' + subtypes + '>>'
                    if extractors is not None:
                        subreturnval += var_name
                        # close iteration
                        extractors.append(
                            indent("};\n", tab * 2 + tab * depth))
                        if in_dict:
                            if i == 0:
                                xtc = "let dict_key = {};\n".format(var_name)
                            elif i == 1:
                                xtc = "let dict_value = {};\n".format(var_name)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                        elif in_list:
                            raise NotImplementedError
                elif type_[0] == 'HashMap':
                    if extractors is not None:
                        name = rnd_var_name(self, arg_names)
                        if in_tuple:
                            tpl_xtc = rnd_var_name(self, arg_names)
                            xtc = "let {name} = {parent}.get_item(*(self.py), {pos})" \
                                + ".unwrap();\n".format(parent=parent,
                                                        pos=i,
                                                        name=tpl_xtc)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                            dic_parent = tpl_xtc
                        elif in_dict:
                            if i == 0:
                                dic_parent = 'key'
                            elif i == 1:
                                dic_parent = 'value'
                        elif in_list:
                            ls_xtc = rnd_var_name(self, arg_names)
                            xtc = "let {name} = e.unwrap();\n".format(
                                name=ls_xtc)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                            dic_parent = ls_xtc
                        else:
                            dic_parent = parent
                        xtc = self._iter_dict.format(
                            parent=dic_parent,
                            var_name=name,)
                        extractors.append(indent(xtc, tab * 2 + tab * depth))
                        new_hm = name
                    else:
                        new_hm = True
                    subtypes, subreturnvalues = self.unwrap_types(
                        type_[1], "", "",
                        in_dict=new_hm,
                        extractors=extractors,
                        arg_names=arg_names,
                        parent=var_name,
                        depth=depth + 1)
                    var_name = new_hm
                    if in_tuple or in_list or in_dict:
                        subtype += 'HashMap<' + subtypes + '>'
                    else:
                        subtype += '<HashMap<' + subtypes + '>>'
                    if extractors is not None:
                        subreturnval += var_name
                        # close iteration
                        xtc = "{var_name}.insert(dict_key, dict_value);\n"\
                            .format(var_name=var_name)
                        extractors.append(indent(xtc, tab * 3 + tab * depth))
                        extractors.append(
                            indent("};\n", tab * 2 + tab * depth))
                        if in_dict:
                            if i == 0:
                                xtc = "let dict_key = {};\n".format(var_name)
                            elif i == 1:
                                xtc = "let dict_value = {};\n".format(var_name)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
                        elif in_list:
                            xtc = "{parent}.push({var_name});\n".format(
                                parent=in_list, var_name=var_name)
                            extractors.append(
                                indent(xtc, tab * 2 + tab * depth))
            elif len(iterable) < 2:
                if in_list or in_dict:
                    subtype += type_
                else:
                    subtype += '<' + type_ + '>'
                if extractors is not None:
                    subreturnval += var_name
            else:
                subtype += type_
                if extractors is not None:
                    subreturnval += var_name
            append_to_sig.append(subtype)
            append_to_val.append(subreturnval)
        return sig.join(append_to_sig), return_val.join(append_to_val)


class RustFuncGen(object):
    ERR_NO_PACKAGE = "no package root found, add an __init__.py file " \
        "to the root of your package"
    ERR_RETURN_TYPE = "function `{}` of module `{}` does not have " \
        "a return type"
    ERR_PARAM_TYPE = "function `{}` of module `{}` lacks type hint for " \
        "one or more parameters"
    ERR_VARORKWARG = "function `{}` of module `{}` cannot take keyword " \
        "or variable arguments"
    ERR_HAS_DEFAULT = "function `{}` of module `{}` cannot define " \
        "default parameter values"
    ERR_INVALID_PARAM = "type of parameter `{}` in function `{}`\n" \
        "of module `{}` is not supported by rustypy,\n" \
        "check the documentation for supported types"
    ERR_NO_MODULE = "the specified python module cannot be found in the" \
        "PYTHONPATH"

    class InvalidType(TypeError):

        def __init__(self, param):
            self.p = param

    def __init__(self, module=False, with_path=None, prefixes=None):
        if isinstance(prefixes, list):
            self.prefixes = prefixes
        elif isinstance(prefixes, str):
            self.prefixes = [prefixes]
        else:
            self.prefixes = ['rust_bind_']
        # get package root for parsing
        if not with_path:
            caller = inspect.stack()[1]
            info = dict(inspect.getmembers(caller.frame))
            path = info["f_globals"]["__file__"]
            path = os.path.abspath(path)
        else:
            path = with_path

        if module is True or inspect.ismodule(module):
            self._ismodule = True
            if inspect.ismodule(module):
                self.root = module
            else:
                if not os.path.exists(path):
                    raise FileNotFoundError(path)
                try:
                    mod = import_module(
                        os.path.basename(path).replace('.py', ''))
                except:
                    raise ImportError(ERR_NO_MODULE)
                self.root = os.path.dirname(path)
        else:
            self._ismodule = False
            # discover the root of the package
            init_f = os.path.join(path, '__init__.py')
            if os.path.exists(init_f):
                path = init_f
            else:
                while os.path.basename(path) != '__init__.py':
                    path = os.path.dirname(path)
                    for f in os.listdir(path):
                        if f == "__init__.py":
                            path = os.path.join(path, f)
                            break
                    if path == os.path.abspath(os.sep):
                        raise FileNotFoundError(self.ERR_NO_PACKAGE)
            self.root = os.path.dirname(path)
            self.pyfiles, self.rsfiles = [], []
            # add all files to process
            for root, _, files in os.walk(self.root, topdown=False):
                for f in files:
                    if f[-3:] == ".py":
                        self.pyfiles.append(os.path.join(root, f))
                    elif f[-3:] == ".rs":
                        self.rsfiles.append(os.path.join(root, f))
        # parse python files function signatures to generate bindings
        self.parse_functions()
        # write the code in a Rust file
        self.dump_to_rust()

    Func = namedtuple('Func',
                      ['name', 'parameters', 'rsreturn', 'pyargs']
                      )
    #Klass = namedtuple('Klass', ['name'])

    def parse_functions(self):
        def inspect_parameters():
            functions, klasses = [], []
            # find out functions and classes for the module
            for name, obj in module.items():
                for prefix in self.prefixes:
                    pl = len(prefix)
                    if inspect.isfunction(obj) \
                            and (name[:pl] == prefix or hasattr(obj, '_bind_to_rust')) \
                            and name not in functions:
                        functions.append(name)
                        self.__no_funcs = False
            if self.__no_funcs:
                del m_dict[imp_statement]
                return
            # introspect parameters for functions and assert they have
            # proper types
            for func in functions:
                func = module[func]
                if func.__name__ == 'rust_bind':
                    continue
                sig = inspect.signature(func)
                if sig.return_annotation is sig.empty:
                    raise TypeError(self.ERR_RETURN_TYPE
                                    .format(func.__name__, imp_statement))
                params = []
                for param in sig.parameters.values():
                    if param.kind == inspect.Parameter.VAR_POSITIONAL \
                            or param.kind == inspect.Parameter.VAR_KEYWORD:
                        raise TypeError(self.ERR_VARORKWARG
                                        .format(func.__name__, imp_statement))
                    if param.annotation is param.empty:
                        raise TypeError(self.ERR_PARAM_TYPE
                                        .format(func.__name__, imp_statement))
                    if param.default is not param.empty:
                        raise TypeError(self.ERR_HAS_DEFAULT
                                        .format(func.__name__, imp_statement))
                    try:
                        parsed_rstypes = self.parse_parameter(param)
                    except RustFuncGen.InvalidType as err:
                        raise TypeError(self.ERR_INVALID_PARAM
                                        .format(err.p.name, func.__name__, imp_statement))
                    params.append((param.name, parsed_rstypes))
                rust_sig = self.Func(
                    name=func.__name__,
                    parameters=params,
                    rsreturn=self.parse_parameter(sig.return_annotation),
                    pyargs=self.parse_parameter(sig.return_annotation,
                                                pytypes=True),
                )
                module_objs.append(rust_sig)

        self.m_dict = m_dict = {}
        if inspect.ismodule(self.root):
            module = self.root.__dict__
            module_objs = m_dict[self.root.__name__] = []
            imp_statement = module["__spec__"].name
            inspect_parameters()
        else:
            root, libname = os.path.split(self.root)
            sys.path.append(root)
            invalidate_caches()
            for f in self.pyfiles:
                self.__no_funcs = True
                # get the absolute import path and dynamically import it
                path, filename = os.path.split(f)
                rel_imp = [libname, None]
                while True:
                    path, tail = os.path.split(path)
                    if path == root or path == os.path.abspath(os.sep):
                        break
                    rel_imp.insert(1, tail)
                rel_imp = [s + '.' for s in rel_imp[:-1]]
                rel_imp.append(filename[:-3])
                imp_statement = "".join(rel_imp)
                module_objs = m_dict[imp_statement] = []
                module = import_module(imp_statement).__dict__
                inspect_parameters()
            sys.path.pop()

    def parse_parameter(self, p, pytypes=False):
        def inner_types(t, curr):
            add = []
            param = False
            if inspect.isclass(t):
                if t is int:
                    if pytypes:
                        param = "PyLong"
                    else:
                        param = "c_long"
                elif t is float or t is Double or t is Float:
                    if pytypes:
                        param = "PyFloat"
                    else:
                        param = "c_double"
                elif t is str:
                    if pytypes:
                        param = "PyString"
                    else:
                        param = "String"
                elif t is bool:
                    if pytypes:
                        param = "PyBool"
                    else:
                        param = "bool"                
                elif issubclass(t, Tuple):
                    if pytypes:
                        curr.append('PyTuple')
                    else:
                        curr.append('tuple')
                    for type_ in t:
                        inner_types(type_, add)
                    param = True
                elif issubclass(t, (list, abc.MutableSequence)):
                    if pytypes:
                        curr.append("PyList")
                    else:
                        curr.append("Vec")
                    for type_ in t.__args__:
                        inner_types(type_, add)
                    param = True
                elif issubclass(t, (dict, abc.MutableMapping)):
                    if pytypes:
                        curr.append("PyDict")
                    else:
                        curr.append("HashMap")
                    for type_ in t.__args__:
                        inner_types(type_, add)
                    param = True
                elif issubclass(t, (set, abc.MutableSet)):
                    raise NotImplementedError("rustypy: support for sets not added yet")
                    if pytypes:
                        curr.append("PySet")
                    else:
                        curr.append("Set")
                    for type_ in t.__args__:
                        inner_types(type_, add)
                    param = True
                elif t.__class__ is typing.GenericMeta:
                    param = "PyObject"
                elif issubclass(t, FunctionType):
                    param = False
            elif t is None:
                if pytypes:
                    param = 'PyNone'
                else:
                    param = 'PyObject::None'
            # check if is a valid type or raise exception
            if not isinstance(param, bool):
                curr.append(param)
            elif not param:
                raise self.InvalidType(p)
            if len(add) > 0:
                curr[-1] = (curr[-1], add)

        try:
            type_ = p.annotation
        except AttributeError:
            type_ = p
        param = []
        inner_types(type_, param)
        return param

    _file_header_info = """
    //! This file has been generated by rustypy and contains bindings for Python.
    //! rustypy version: {rustypy_ver}
    //! Python implementation build version: {python_ver}

    // IMPORTANT: This file will be replaced on new rustypy calls.
    // Don't write on it directly, rather call functions from an other file.
    """.format(
        rustypy_ver=rustypy_ver,
        python_ver=sys.implementation.cache_tag
    )

    _file_header_funcs = """
    #![allow(
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        unused_imports,
        unused_variables)]

    use libc::{c_long, c_double};
    use cpython::{Python, ToPyObject, FromPyObject, ObjectProtocol,
                  PythonObjectWithCheckedDowncast, PyObject, PyModule, PyErr,
                  PyDict, NoArgs};
    use std::collections::HashMap;

    fn handle_import_error(err: PyErr) {
        println!("failed to load Python module, reason:
                 {}", err.pvalue.unwrap());
    }
    """

    _mod_manager = Template("""
    /// Python module manager
    pub struct PyModules<'a> {
$mod_list
    }

    impl<'a> PyModules<'a> {
        pub fn new(py: &'a Python) -> PyModules<'a> {
            PyModules {
$mod_list_init
            }
        }
    }
    """)

    _mod_struct = Template("""
    /// Binds for Python module `$m_name`
    pub struct $name<'a> {
        module: PyModule,
        py: &'a Python<'a>
    }

    impl<'a> $name<'a> {
        /// Loads the module instance to the interpreter
        pub fn new(py: &'a Python) -> Option<$name<'a>> {
            let module = py.import("$import_path");
            match module {
                Ok(m) => Some(
                    $name {
                        module: m,
                        py: py
                    }),
                Err(exception) => {
                    handle_import_error(exception);
                    None
                }
            }
        }
        $functions
    }
    """)

    _folder_struct = Template("""
    /// Struct for folder `$f_name`
    pub struct $name<'a> {
$fields
    }

    impl<'a> $name<'a> {
        /// Loads the module instance to the interpreter
        pub fn new(py: &'a Python) -> Option<$name<'a>> {
            Some($name {
$build
            })
        }
    }
    """)

    class PckgStruct:

        def __init__(
                self, folder=True, name=None, origin=False, ismodule=False):
            if folder:
                self.folder = True
                self.name = name
            else:
                self.folder = False
            if origin:
                self.origin = True
            else:
                self.origin = False
            if ismodule:
                self.ismodule = True
            else:
                self.ismodule = False
            self._subfolders, self._submods = [], []

        def add_mod(self, mod):
            x = RustFuncGen.PckgStruct(folder=False)
            x.mod = mod
            self._submods.append(x)

        def add_folder(self, name):
            x = RustFuncGen.PckgStruct(name=name)
            self._subfolders.append(x)
            return x

        def walk(self, path, mod):
            if len(path) == 0:
                self.add_mod(mod)
            elif len(path) == 1:
                self.add_mod(mod)
            else:
                next_f, done = path[0], False
                for f in self._subfolders:
                    if f.name == next_f:
                        f.walk(path[1:], mod)
                        done = True
                        break
                if not done:
                    f = self.add_folder(next_f)
                    f.walk(path[1:], mod)

        _field = tab * 2 + "pub {n}: {rn}<'a>,\n"
        _build = tab * 4 + '{n}: {rn}::new(&py).unwrap(),\n'

        def write_structs(self, f, struct_names={}):
            if self.ismodule:
                self.write_single_module(f)
                return
            if self.origin:
                rn = rnd_var_name(None, struct_names.values())
                struct_names[self] = rn
            field, build = self._field, self._build
            if self.folder:
                # process subfolders
                fields, builds = [], []
                for x in self._subfolders:
                    rn = rnd_var_name(None, struct_names.values())
                    struct_names[x] = rn
                    fields.append(field.format(n=x.name, rn=rn))
                    builds.append(build.format(n=x.name, rn=rn))
                    x.write_structs(f)
                # process submodules
                for x in self._submods:
                    rn = rnd_var_name(None, struct_names.values())
                    struct_names[x] = rn
                    fields.append(field.format(n=x.mod.name[-1], rn=rn))
                    builds.append(build.format(n=x.mod.name[-1], rn=rn))
                    x.write_structs(f)
                builds, fields = "".join(builds), "".join(fields)
                code = dedent(RustFuncGen._folder_struct.substitute(
                    name=struct_names[self],
                    fields=fields,
                    build=builds,
                    f_name=self.name))
                f.write(code)
            else:
                imp_path = "".join([x + '.' for x in self.mod.name])[:-1]
                code = dedent(RustFuncGen._mod_struct.substitute(
                    m_name=self.mod.name[-1],
                    name=struct_names[self],
                    import_path=imp_path,
                    functions=self.mod._generated_functions.getvalue()))
                f.write(code)
            if self.origin:
                mod_list, mod_list_init = [], []
                for x in self._subfolders:
                    n = field.format(n=x.name, rn=struct_names[x])
                    mod_list.append(n)
                    n = build.format(n=x.name, rn=struct_names[x])
                    mod_list_init.append(n)
                for x in self._submods:
                    n = field.format(n=x.mod.name[-1], rn=struct_names[x])
                    mod_list.append(n)
                    n = build.format(n=x.mod.name[-1], rn=struct_names[x])
                    mod_list_init.append(n)
                mod_list = "".join(mod_list)
                mod_list_init = "".join(mod_list_init)
                code = dedent(RustFuncGen._mod_manager.substitute(
                    mod_list=mod_list, mod_list_init=mod_list_init))
                f.write(code)

        def write_single_module(self, f):
            if len(self._subfolders) == 0:
                mod = self._submods[0].mod
            else:
                folder = self._subfolders[0]
                mod = folder._submods[0].mod
            mod_name = mod.name[-1]
            code = dedent(RustFuncGen._mod_struct.substitute(
                m_name=mod_name,
                name=mod_name,
                import_path=mod_name,
                functions=mod._generated_functions.getvalue()))
            f.write(code)
            mod_list = self._field.format(n=mod_name, rn=mod_name)
            mod_list_init = self._build.format(n=mod_name, rn=mod_name)
            code = dedent(RustFuncGen._mod_manager.substitute(
                mod_list=mod_list, mod_list_init=mod_list_init))
            f.write(code)

    def dump_to_rust(self):
        if inspect.ismodule(self.root):
            dir_ = os.path.split(self.root.__file__)[0]
            pckg_struct = self.PckgStruct(ismodule=True)
            for mod, data in self.m_dict.items():
                ModuleStruct(mod, data, pckg_struct)
        else:
            ori = os.path.split(self.root)[1]
            dir_ = self.root
            pckg_struct = self.PckgStruct(name=ori, origin=True)
            for mod, data in self.m_dict.items():
                ModuleStruct(mod, data, pckg_struct)
        file = os.path.join(dir_, 'rustypy_pybind.rs')
        file_header = "".join(
            [self._file_header_info, self._file_header_funcs])
        with open(file, 'w', encoding="UTF-8") as f:
            f.write(dedent(self._file_header_info))
            f.write(dedent(self._file_header_funcs))
            pckg_struct.write_structs(f)
        return

# ==================== #
#   Helper Functions   #
# ==================== #


def rust_bind(fn: FunctionType) -> FunctionType:
    if hasattr(fn, '_bind_to_rust'):
        raise AttributeError(
            "the attribute name `_bind_to_rust` is reserved for binding "
            "functions of RustyPy")
    fn._bind_to_rust = True
    return fn


def rnd_var_name(self=None, cmp=[]):
    var_name = ""
    if self:
        while not var_name or var_name in cmp or var_name in self._var_names:
            var_name = var_name.join(random.choice(ascii_letters)
                                     for _ in range(6))
    else:
        while not var_name or var_name in cmp:
            var_name = var_name.join(random.choice(ascii_letters)
                                     for _ in range(6))
    return var_name


def bind_py_pckg_funcs(prefixes=None):
    caller = inspect.stack()[1]
    info = dict(inspect.getmembers(caller.frame))
    path = info["f_globals"]["__file__"]
    path = os.path.abspath(path)
    RustFuncGen(with_path=path, prefixes=prefixes)
