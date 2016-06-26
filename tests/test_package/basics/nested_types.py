import typing as typ
import collections.abc as abc
from rustypy import rust_bind

# generics:
T = typ.TypeVar('A', int, str)
@rust_bind
def generic1(g_arg: T) -> T:

    assert isinstance(g_arg, int) or isinstance(g_arg, str), \
        'provided argument is not an int or str'
    return g_arg

@rust_bind
def generic2(g_arg: typ.List[T]) -> typ.List[T]:
    if g_arg[0] != 0:
        raise AssertionError
    if g_arg[1] != 'second':
        raise AssertionError
    out = ['success']
    return out

# containers/mappings:
@rust_bind
def dict1(dict_arg: typ.Dict[str, int]) \
    -> typ.Dict[str, int]:
    for k, v in dict_arg.items():
        dict_arg[k] = v + 1
    return dict_arg

K = typ.Tuple[str, bool]
U = typ.Dict[str, K]
@rust_bind
def dict2(dict_arg: U) -> U:
    return dict_arg

J = typ.Tuple[float, bool]
U = typ.List[J]
@rust_bind
def list1(ls_arg: U) \
    -> typ.List[str]:
    for e in ls_arg:
        if not isinstance(e[0], float):
            raise AssertionError
        if not isinstance(e[1], bool):
            raise AssertionError
    out_ls = ['passed']
    return out_ls

# nested types:
@rust_bind
def cmpd_tuple(tup_arg1: typ.Tuple[int, J]) -> typ.Tuple[int, K, float]:
    out = (1, ('passed', True), 0.0)
    return out

X = typ.List[typ.Tuple[K, T]]
U = typ.List[typ.Tuple[int, bool]]
@rust_bind
def cmpd_list_and_tuple(ls_arg: X) -> U:
    out = []
    for i, e in enumerate(ls_arg):
        if not isinstance(e, tuple):
            raise AssertionError("list value not a tuple")
        if isinstance(e[1], str):
            out.append((i, True))
        elif isinstance(e[1], int):
            out.append((i, False))
        else:
            raise AssertionError("value is neither a string or an integer")
    return out

U = typ.List[typ.Tuple[int, bool]]
@rust_bind
def cmpd_list(arg1: U, arg2: typ.List[int]) \
        -> typ.List[typ.Tuple[typ.List[int], float]]:
    for e in arg1:
        assert isinstance(e[0], int)
        assert isinstance(e[1], bool)
    for e in arg2:
        assert isinstance(e, int)
    out = [([1], 1.0)]
    return out

U = typ.Dict[int, K]
@rust_bind
def cmpd_dict() -> typ.Dict[str, U]:
    d = {'passed': {0: ('passed', True)}}
    return d

@rust_bind
def cmpd_list_and_dict() -> typ.List[U]:
    ls = [{0: ('passed', True)}]
    return ls

@rust_bind
def cmpd_dict_and_ls() -> typ.Dict[int, typ.List[float]]:
    d = {0: [0.0, 1.0, 2.0, 3.0]}
    return d

# not supported yet:
"""
U = typ.Set[K]
def rust_bind_set1(set_arg: typ.Set[K]) \
    -> typ.Set[K]:
    return set_arg
def rust_bind_set2(set_arg: U) -> U:
    return set_arg
"""
