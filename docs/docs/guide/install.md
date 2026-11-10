# Install PhytKit

## From PyPI

The quickest way to get PhytKit is the Python package, which also installs
the `pk` command. It needs Python 3.9 or newer, and has prebuilt wheels for
Linux, macOS and Windows:

```bash
pip install phytkit
pk --help
```

## From source

Building from source needs:

- **Rust 1.88 or newer**: install it from [rustup.rs](https://rustup.rs).
- **For the Python package:** Python 3.9 or newer.

Get the source with
`git clone https://github.com/PlantSimulationLab/PhytKit.git`. All commands
below run from the top folder of the source, the one with `Cargo.toml` in it.

### The `pk` command

```bash
cargo build --release
./target/release/pk --help
```

The first build downloads PhytKit's Rust dependencies and takes a few
minutes. Later builds are quick.

To run `pk` from anywhere, install it onto your PATH:

```bash
cargo install --path crates/cli
pk --help
```

The rest of these docs write `pk`. If you didn't install it, use
`./target/release/pk` instead.

### The Python package

The Python package is built with [maturin](https://www.maturin.rs), which
compiles the Rust code into a Python extension:

```bash
python3 -m venv .venv
.venv/bin/pip install maturin
VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop --release -m crates/py/Cargo.toml
.venv/bin/python -c "import phytkit; print(phytkit.__version__)"
```

`maturin develop` installs into the virtual environment named by
`VIRTUAL_ENV`. Use any environment you like; the commands above make a new
one.

!!! warning "Using Conda"
    If a Conda environment is active, maturin may install into Conda instead
    of your virtual environment, or refuse to choose. Run it with
    `env -u CONDA_PREFIX` in front:

    ```bash
    env -u CONDA_PREFIX VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop --release -m crates/py/Cargo.toml
    ```

The package also installs a `pk` command into the environment, which behaves
like the Rust one.

## Check it works

The sample files are in the `examples/` folder of the
[source repository](https://github.com/PlantSimulationLab/PhytKit); they are
not part of the pip package. If you installed from PyPI, clone the repository
(or download `examples/aci.csv`) and run these from its top folder.

```bash
pk describe fvcb
pk fit photosynthesis examples/aci.csv --model fvcb
```

The first prints the FvCB model's parameters. The second fits the sample
A–Ci curves and prints the result. [Your first fit](first-fit.md) explains
it.
