# /// script
# dependencies = [
#   "typer",
#   "scikit-learn",
#   "numpy",
#   "pandas",
#   "fastparquet",
#   "lightgbm",
# ]
# ///

"""Top-level experiments CLI.

This CLI exposes a train subcommand and wraps common Cargo workflows
used in this repository.
"""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Optional

import typer

from commands.train import app as train_app

app = typer.Typer(
    add_completion=False,
    no_args_is_help=True,
    help="Experiments CLI with training and Cargo-backed commands.",
)
app.add_typer(train_app, name="train", help="ML training commands")

ROOT_DIR = Path(__file__).resolve().parents[1]
FBOOL_EXPERIMENT_BINS = {
    "npn-create",
    "compute-metrics",
    "bayes-optimal",
    "orthogonality",
}


PATH_FLAGS = {
    "--input",
    "--output",
    "--output-path",
    "--npn-table",
    "--output-csv",
    "--output-mi-csv",
}
PATH_SHORT_FLAGS = {"-i", "-o"}


def _normalize_path_value(value: str, base_dir: Path) -> str:
    if value == "" or value == "-":
        return value

    path = Path(value)
    if path.is_absolute():
        return value

    return str((base_dir / path).resolve())


def normalize_experiments_bin_args(extra_args: list[str], invocation_dir: Path) -> list[str]:
    normalized: list[str] = []
    pending_path_flag: Optional[str] = None

    for token in extra_args:
        if pending_path_flag is not None:
            normalized.append(_normalize_path_value(token, invocation_dir))
            pending_path_flag = None
            continue

        if token in PATH_FLAGS or token in PATH_SHORT_FLAGS:
            normalized.append(token)
            pending_path_flag = token
            continue

        if token.startswith("--") and "=" in token:
            flag, raw_value = token.split("=", 1)
            if flag in PATH_FLAGS:
                normalized.append(f"{flag}={_normalize_path_value(raw_value, invocation_dir)}")
            else:
                normalized.append(token)
            continue

        normalized.append(token)

    return normalized


def run_cargo(args: list[str], cwd: Optional[Path] = None) -> None:
    cmd = ["cargo", *args]
    exec_cwd = cwd or ROOT_DIR
    print(f"[cargo] cwd={exec_cwd}")
    print(f"[cargo] command={' '.join(cmd)}")
    proc = subprocess.run(cmd, cwd=exec_cwd, check=False)
    if proc.returncode != 0:
        raise typer.Exit(code=proc.returncode)


@app.command("build")
def cmd_build(
    release: bool = typer.Option(True, "--release/--debug", help="Build profile"),
    package: Optional[str] = typer.Option(None, "--package", "-p", help="Optional package name"),
) -> None:
    """Run cargo build for workspace or one package."""
    args = ["build"]
    if release:
        args.append("--release")
    if package:
        args.extend(["-p", package])
    run_cargo(args)


@app.command("test")
def cmd_test(
    package: Optional[str] = typer.Option(None, "--package", "-p", help="Optional package name"),
) -> None:
    """Run cargo test for workspace or one package."""
    args = ["test", "--all"]
    if package:
        args = ["test", "-p", package]
    run_cargo(args)


@app.command("clippy")
def cmd_clippy() -> None:
    """Run cargo clippy for all targets/features."""
    run_cargo(["clippy", "--all-targets", "--all-features"])


@app.command(
    "fbool-cli",
    context_settings={"allow_extra_args": True, "ignore_unknown_options": True},
)
def cmd_fbool_cli(ctx: typer.Context) -> None:
    """Run fbool CLI through cargo run -p fbool-cli --bin fbool -- ..."""
    run_cargo(["run", "-p", "fbool-cli", "--bin", "fbool", "--", *ctx.args])


@app.command(
    "experiments-bin",
    context_settings={"allow_extra_args": True, "ignore_unknown_options": True},
)
def cmd_experiments_bin(
    ctx: typer.Context,
    bin_name: str = typer.Argument(..., help="fbool-experiments binary name"),
    release: bool = typer.Option(True, "--release/--debug", help="Run release or debug binary"),
) -> None:
    """Run a fbool-experiments bin with cargo run."""
    if bin_name not in FBOOL_EXPERIMENT_BINS:
        allowed = ", ".join(sorted(FBOOL_EXPERIMENT_BINS))
        raise typer.BadParameter(f"Unknown bin '{bin_name}'. Allowed: {allowed}")

    invocation_dir = Path.cwd()
    extra_args = normalize_experiments_bin_args(ctx.args, invocation_dir)

    args = ["run", "-p", "fbool-experiments", "--bin", bin_name]
    if release:
        args.append("--release")
    args.extend(["--", *extra_args])
    run_cargo(args)


if __name__ == "__main__":
    app()
