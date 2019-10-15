import functools
import sys
import typing


def type_checkers(func):
    prev_to_37 = sys.version_info[0:2] <= (3, 6)
    @functools.wraps(func)
    def checker(*args, **kwargs):
        if prev_to_37:
            def other_type_checker(arg_t):
                return False

            def is_generic(arg_t):
                return arg_t.__class__ is typing.GenericMeta

            checkers = {
                "other_type": other_type_checker,
                "generic": is_generic,
            }
        else:
            def other_type_checker(arg_t):
                return False

            def is_generic(arg_t):
                return arg_t.__class__ is typing._GenericAlias

            checkers = {
                "other_type": other_type_checker,
                "generic": is_generic,
            }

        kwargs.update(checkers)
        return func(*args, **kwargs)
    return checker
