import typing
from rustypy import rust_bind
from rustypy.rswrapper import Tuple, List, Dict

# generics:
T = typing.TypeVar('A', int, str)


@rust_bind
def generic1(g_arg: T) -> T:
    assert isinstance(g_arg, int) or isinstance(g_arg, str), \
        'provided argument is not an int or str'
    return g_arg


@rust_bind
def generic2(g_arg: List[T]) -> List[T]:
    if g_arg[0] != 0:
        raise AssertionError
    if g_arg[1] != 'second':
        raise AssertionError
    out = ['success']
    return out

# containers/mappings:


@rust_bind
def dict1(dict_arg: Dict[str, int]) \
        -> Dict[str, int]:
    for k, v in dict_arg.items():
        dict_arg[k] = v + 1
    return dict_arg

K = Tuple[str, bool]
U = Dict[str, K]


@rust_bind
def dict2(dict_arg: U) -> U:
    return dict_arg

J = Tuple[float, bool]
U = List[J]


@rust_bind
def list1(ls_arg: U) \
        -> List[str]:
    for e in ls_arg:
        if not isinstance(e[0], float):
            raise AssertionError
        if not isinstance(e[1], bool):
            raise AssertionError
    out_ls = ['passed']
    return out_ls

# nested types:


@rust_bind
def cmpd_tuple(tup_arg1: Tuple[int, J]) -> Tuple[int, K, float]:
    out = (1, ('passed', True), 0.0)
    return out

X = List[Tuple[K, T]]
U = List[Tuple[int, bool]]


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

U = List[Tuple[int, bool]]


@rust_bind
def cmpd_list(arg1: U, arg2: List[int]) \
        -> List[Tuple[List[int], float]]:
    for e in arg1:
        assert isinstance(e[0], int)
        assert isinstance(e[1], bool)
    for e in arg2:
        assert isinstance(e, int)
    out = [([1], 1.0)]
    return out

U = Dict[int, K]


@rust_bind
def cmpd_dict() -> Dict[str, U]:
    d = {'passed': {0: ('passed', True)}}
    return d


@rust_bind
def cmpd_list_and_dict() -> List[U]:
    ls = [{0: ('passed', True)}]
    return ls


@rust_bind
def cmpd_dict_and_ls() -> Dict[int, List[float]]:
    d = {0: [0.0, 1.0, 2.0, 3.0]}
    return d

# not supported yet:
"""
U = typing.Set[K]
def rust_bind_set1(set_arg: typing.Set[K]) \
    -> typing.Set[K]:
    return set_arg
def rust_bind_set2(set_arg: U) -> U:
    return set_arg
"""
