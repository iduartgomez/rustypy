import functools
import sys
import typing
from collections import abc

prev_to_37 = sys.version_info[0:2] <= (3, 6)
if prev_to_37:
    def is_map_like(arg_t):
        if hasattr(arg_t, "__origin__"):
            return issubclass(arg_t.__origin__, (dict, abc.MutableMapping))
        return issubclass(arg_t.__class__, (dict, abc.MutableMapping))


    def is_seq_like(arg_t):
        if hasattr(arg_t, "__origin__"):
            return issubclass(arg_t.__origin__, (list, abc.MutableSequence))
        return issubclass(arg_t.__class__, (list, abc.MutableSequence))


    def is_generic(arg_t):
        return arg_t.__class__ is typing.GenericMeta
else:
    def is_map_like(arg_t):
        if hasattr(arg_t, "__origin__"):
            return issubclass(arg_t.__origin__, (dict, abc.MutableMapping))
        return False


    def is_seq_like(arg_t):
        if hasattr(arg_t, "__origin__"):
            return issubclass(arg_t.__origin__, (list, abc.MutableSequence))
        return False


    def is_generic(arg_t):
        if hasattr(arg_t, "__origin__"):
            return arg_t.__origin__ is typing.Generic
        return False


def type_checkers(func):
    @functools.wraps(func)
    def checker(*args, **kwargs):

        checkers = {
            "map_like": is_map_like,
            "seq_like": is_seq_like,
            "generic": is_generic,
        }

        kwargs.update(checkers)
        return func(*args, **kwargs)
    return checker
