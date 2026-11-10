"""The Python keyword wrappers spell out every option of their pk command, so an agent can
read them from the signature and docstring. These tests keep them equal to the COMMANDS
table (`describe()["commands"]`) and the compiled functions to their stubs."""
import ast
import inspect
from pathlib import Path

import pytest

import phytkit as pk

WRAPPERS = {"fit": pk.fit, "li600 correct": pk.li600_correct, "li600 estimate": pk.li600_estimate, "concat": pk.concat}


def command_options(name):
    cmd = next(c for c in pk.describe()["commands"] if c["name"] == name)
    return [o["name"][2:].replace("-", "_") for o in cmd["options"]]


@pytest.mark.parametrize("command", list(WRAPPERS))
def test_wrapper_keywords_equal_the_command_options_in_order(command):
    fn = WRAPPERS[command]
    sig = inspect.signature(fn)
    kw = [n for n, p in sig.parameters.items() if p.kind is inspect.Parameter.KEYWORD_ONLY]
    assert kw == command_options(command)
    # A default of None or False means "leave the option out", so the command's default applies.
    for n in kw:
        assert sig.parameters[n].default in (None, False), n
    # Unknown keywords still reach pk, which rejects them with a suggestion.
    assert any(p.kind is inspect.Parameter.VAR_KEYWORD for p in sig.parameters.values())
    # Every option is explained in the docstring.
    for n in kw:
        assert f"{n}:" in fn.__doc__ or f" {n}" in fn.__doc__, f"{fn.__name__} docstring lacks {n}"


def test_switches_are_false_by_default_and_named_options_reach_pk():
    res = pk.fit("photosynthesis", Path(__file__).resolve().parents[3] / "examples" / "aci.csv",
                 ci_based=True, max_iter=5, fit=["Vcmax25", "Jmax25"])
    assert res["ciBased"] is True and res["fitted"] == ["Vcmax25", "Jmax25"]


def stub_functions():
    tree = ast.parse((Path(pk.__file__).parent / "_phytkit.pyi").read_text())
    return {n.name: n for n in tree.body if isinstance(n, ast.FunctionDef)}


def test_stub_lists_every_compiled_function_with_its_parameters():
    stubs = stub_functions()
    compiled = {n for n in dir(pk._phytkit) if callable(getattr(pk._phytkit, n)) and not n.startswith("__") and n != "PhytKitError"}
    assert set(stubs) == compiled
    for name, node in stubs.items():
        want = [a.arg for a in node.args.args + node.args.kwonlyargs]
        got = list(inspect.signature(getattr(pk._phytkit, name)).parameters)
        assert want == got, name


@pytest.mark.parametrize("name", ["fit_arrays", "fit_many"])
def test_array_wrappers_match_the_compiled_functions(name):
    want = inspect.signature(getattr(pk._phytkit, name)).parameters
    got = inspect.signature(getattr(pk, name)).parameters
    assert list(got) == list(want)
