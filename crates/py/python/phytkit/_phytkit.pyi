"""Type stubs for the compiled module. The keyword wrappers in phytkit/__init__.py are plain
Python; these are the functions implemented in Rust (crates/py/src/lib.rs). test_signatures.py
checks the names and parameters against the compiled module."""
from typing import Any, Mapping, Sequence

__version__: str

class PhytKitError(Exception):
    code: str
    hint: str | None
    usage: bool
    help: str | None

def run(argv: list[str]) -> dict[str, Any] | str: ...
def main(argv: list[str]) -> tuple[str, int]: ...
def models() -> list[str]: ...
def fit_arrays(
    model_name: str,
    x: Mapping[str, Sequence[float]],
    y: Sequence[float],
    *,
    fit_params: Sequence[str] | None = None,
    init: Mapping[str, float] | None = None,
    max_iter: int | None = None,
    curves: Sequence[Sequence[int]] | None = None,
    constraints: Sequence[str] | None = None,
    constraint_weights: Mapping[str, float] | None = None,
) -> dict[str, Any]: ...
def fit_many(
    model_name: str,
    datasets: Sequence[tuple[Mapping[str, Sequence[float]], Sequence[float]]],
    *,
    fit_params: Sequence[str] | None = None,
    init: Mapping[str, float] | None = None,
    max_iter: int | None = None,
    constraints: Sequence[str] | None = None,
    constraint_weights: Mapping[str, float] | None = None,
    threads: int | None = None,
) -> list[dict[str, Any]]: ...
def predict(model_name: str, params_: Mapping[str, float], x: Mapping[str, Sequence[float]]) -> list[float]: ...
def build_info() -> dict[str, Any]: ...
def _set_skills_dir(path: str) -> None: ...
