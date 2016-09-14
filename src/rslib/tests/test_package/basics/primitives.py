import typing as typ

# primitives:


def rust_bind_int_func(int_arg: int) -> int:
    rval = int_arg + 1
    return rval


def rust_bind_float_func(float_arg: float) -> float:
    rval = float_arg + 0.5
    return rval


def rust_bind_str_func(str_arg: str) -> str:
    rval = str_arg + "added this in Python!"
    return rval


def rust_bind_bool_func(bool_arg: bool) -> bool:
    rval = False
    return rval


def rust_bind_tuple1(tup_arg: typ.Tuple[str, int]) -> typ.Tuple[str, int]:
    output = list(tup_arg)
    output[0] += "added this in Python!"
    output[1] += 10
    output = tuple(output)
    return output

K = typ.Tuple[str, bool]


def rust_bind_tuple2(tup_arg: K) -> K:
    output = list(tup_arg)
    output[0] += "added this in Python!"
    if not output[1]:
        raise AssertionError(output[1])
    else:
        output[1] = False
    output = tuple(output)
    return output

J = typ.Tuple[float, bool]


def rust_bind_tuple3(tup_arg1: float, tup_arg2: bool) -> J:
    out_arg1 = tup_arg1 + 0.5
    if not tup_arg2:
        raise AssertionError(tup_arg2)
    else:
        out_arg2 = False
    return out_arg1, out_arg2
