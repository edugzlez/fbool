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

"""Train and compare ML models on metric parquet files with a Typer CLI.

Main features:
- Subcommands per model with key hyperparameters
- Binary and multiclass targets
- Weighted training when model fit supports sample_weight
- Weighted evaluation using count-based sample weights
"""

from __future__ import annotations

import inspect
import random
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Optional

import numpy as np
import pandas as pd
import typer
from sklearn.ensemble import ExtraTreesClassifier, GradientBoostingClassifier, RandomForestClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import (
    accuracy_score,
    balanced_accuracy_score,
    f1_score,
    precision_score,
    recall_score,
)
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import GaussianNB
from sklearn.neighbors import KNeighborsClassifier
from sklearn.neural_network import MLPClassifier
from sklearn.svm import LinearSVC
from sklearn.tree import DecisionTreeClassifier

DEFAULT_MODELS = [
    "mlp",
    "decision_tree",
    "random_forest",
    "extra_trees",
    "gradient_boosting",
    "logistic_regression",
    "linear_svm",
    "knn",
    "naive_bayes",
    "lightgbm",
]

app = typer.Typer(
    add_completion=False,
    no_args_is_help=True,
    help="Train and compare models from parquet metrics with weighted evaluation.",
)


@dataclass
class CommonConfig:
    input: Path
    target: str
    count_col: str
    binary: bool
    threshold: float
    metrics: Optional[str]
    test_size: float
    seed: int
    output_csv: Optional[Path]
    verbose: bool
    train_verbose: int
    lgb_log_period: int
    normalize: bool
    max_rows: Optional[int]


@dataclass
class PreparedData:
    x_train: np.ndarray
    x_test: np.ndarray
    y_train: np.ndarray
    y_test: np.ndarray
    w_train: np.ndarray
    w_test: np.ndarray
    mode_text: str
    feature_cols: list[str]
    n_classes: int
    rows_used: int


@dataclass
class RunResult:
    model: str
    ok: bool
    train_weighted: bool
    fit_seconds: float
    accuracy_w: float | None = None
    f1_macro_w: float | None = None
    precision_macro_w: float | None = None
    recall_macro_w: float | None = None
    balanced_accuracy_w: float | None = None
    f1_binary_w: float | None = None
    error: str | None = None
    diagnostics: list[str] | None = None


def parse_csv_list(value: str | None) -> list[str]:
    if value is None:
        return []
    parts = [x.strip() for x in value.split(",")]
    return [x for x in parts if x]


def parse_hidden_layers(value: str) -> tuple[int, ...]:
    parts = [x.strip() for x in value.split(",") if x.strip()]
    if not parts:
        raise typer.BadParameter("--hidden-layers must contain at least one integer, e.g. 64,32")
    try:
        layers = tuple(int(x) for x in parts)
    except ValueError as exc:
        raise typer.BadParameter("--hidden-layers must be a comma-separated list of integers") from exc
    if any(x <= 0 for x in layers):
        raise typer.BadParameter("--hidden-layers values must be positive")
    return layers


def parse_float_csv(value: str) -> tuple[float, ...]:
    parts = [x.strip() for x in value.split(",") if x.strip()]
    if not parts:
        raise typer.BadParameter("expected a comma-separated list of floats")
    try:
        values = tuple(float(x) for x in parts)
    except ValueError as exc:
        raise typer.BadParameter("expected a comma-separated list of floats") from exc
    return values


def vlog(enabled: bool, message: str) -> None:
    if enabled:
        print(f"[verbose] {message}")


def supports_sample_weight(estimator: Any) -> bool:
    try:
        sig = inspect.signature(estimator.fit)
    except (ValueError, TypeError):
        return False
    return "sample_weight" in sig.parameters


def make_default_model(name: str, seed: int, n_classes: int, train_verbose: int) -> Any:
    if name == "mlp":
        return MLPClassifier(
            hidden_layer_sizes=(64, 32),
            max_iter=400,
            random_state=seed,
            verbose=(train_verbose > 0),
        )
    if name == "decision_tree":
        return DecisionTreeClassifier(random_state=seed)
    if name == "random_forest":
        return RandomForestClassifier(
            n_estimators=300,
            random_state=seed,
            n_jobs=-1,
            verbose=train_verbose,
        )
    if name == "extra_trees":
        return ExtraTreesClassifier(
            n_estimators=300,
            random_state=seed,
            n_jobs=-1,
            verbose=train_verbose,
        )
    if name == "gradient_boosting":
        return GradientBoostingClassifier(random_state=seed, verbose=train_verbose)
    if name == "logistic_regression":
        if n_classes > 2:
            return LogisticRegression(
                max_iter=2000,
                random_state=seed,
                multi_class="multinomial",
                verbose=train_verbose,
            )
        return LogisticRegression(max_iter=2000, random_state=seed, verbose=train_verbose)
    if name == "linear_svm":
        return LinearSVC(random_state=seed, verbose=train_verbose)
    if name == "knn":
        return KNeighborsClassifier(n_neighbors=25)
    if name == "naive_bayes":
        return GaussianNB()
    if name == "lightgbm":
        try:
            import lightgbm as lgb  # type: ignore
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("lightgbm is not installed. Use uv run --with lightgbm ...") from exc

        objective = "binary" if n_classes == 2 else "multiclass"
        params = {
            "objective": objective,
            "n_estimators": 500,
            "learning_rate": 0.05,
            "num_leaves": 31,
            "subsample": 0.9,
            "colsample_bytree": 0.9,
            "random_state": seed,
            "n_jobs": -1,
            "verbose": (1 if train_verbose > 0 else -1),
        }
        if n_classes > 2:
            params["num_class"] = n_classes
        return lgb.LGBMClassifier(**params)

    raise ValueError(f"Unknown model: {name}")


def make_table(headers: list[str], rows: list[list[str]]) -> str:
    widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            widths[i] = max(widths[i], len(cell))

    def fmt_row(cells: list[str]) -> str:
        return " | ".join(cell.ljust(widths[i]) for i, cell in enumerate(cells))

    sep = "-+-".join("-" * w for w in widths)
    parts = [fmt_row(headers), sep]
    parts.extend(fmt_row(r) for r in rows)
    return "\n".join(parts)


def float_or_dash(x: float | None, ndigits: int = 4) -> str:
    if x is None:
        return "-"
    return f"{x:.{ndigits}f}"


def evaluate_predictions(
    y_true: np.ndarray,
    y_pred: np.ndarray,
    sample_weight: np.ndarray,
    is_binary: bool,
) -> dict[str, float]:
    out: dict[str, float] = {}
    out["accuracy_w"] = float(accuracy_score(y_true, y_pred, sample_weight=sample_weight))
    out["f1_macro_w"] = float(
        f1_score(y_true, y_pred, average="macro", sample_weight=sample_weight, zero_division=0)
    )
    out["precision_macro_w"] = float(
        precision_score(
            y_true,
            y_pred,
            average="macro",
            sample_weight=sample_weight,
            zero_division=0,
        )
    )
    out["recall_macro_w"] = float(
        recall_score(
            y_true,
            y_pred,
            average="macro",
            sample_weight=sample_weight,
            zero_division=0,
        )
    )
    out["balanced_accuracy_w"] = float(
        balanced_accuracy_score(y_true, y_pred, sample_weight=sample_weight)
    )

    if is_binary:
        out["f1_binary_w"] = float(
            f1_score(y_true, y_pred, average="binary", sample_weight=sample_weight, zero_division=0)
        )

    return out


def top_features(feature_names: list[str], importances: Any, top_k: int = 3) -> str:
    try:
        arr = np.asarray(importances, dtype=float)
    except Exception:
        return "n/a"

    if arr.size == 0:
        return "n/a"

    k = min(top_k, arr.size)
    idx = np.argsort(arr)[::-1][:k]
    pairs = [f"{feature_names[i]}={arr[i]:.4f}" for i in idx]
    return ", ".join(pairs)


def model_specific_diagnostics(model_name: str, model: Any, feature_names: list[str]) -> list[str]:
    diag: list[str] = []

    if model_name == "decision_tree":
        depth = getattr(model, "get_depth", lambda: None)()
        leaves = getattr(model, "get_n_leaves", lambda: None)()
        diag.append(f"depth={depth}, leaves={leaves}")
        if hasattr(model, "feature_importances_"):
            diag.append(f"top_features: {top_features(feature_names, model.feature_importances_)}")

    elif model_name in {"random_forest", "extra_trees"}:
        n_estimators = len(getattr(model, "estimators_", []))
        diag.append(f"estimators={n_estimators}")
        estimators = getattr(model, "estimators_", [])
        if estimators:
            depths = [est.get_depth() for est in estimators[: min(50, len(estimators))]]
            diag.append(f"avg_depth(sampled)={float(np.mean(depths)):.2f}")
        if hasattr(model, "feature_importances_"):
            diag.append(f"top_features: {top_features(feature_names, model.feature_importances_)}")

    elif model_name == "gradient_boosting":
        n_estimators = getattr(model, "n_estimators_", None)
        train_score = getattr(model, "train_score_", None)
        diag.append(f"n_estimators_={n_estimators}")
        if train_score is not None and len(train_score) > 0:
            diag.append(f"final_train_deviance={float(train_score[-1]):.6f}")
        if hasattr(model, "feature_importances_"):
            diag.append(f"top_features: {top_features(feature_names, model.feature_importances_)}")

    elif model_name == "mlp":
        n_iter = getattr(model, "n_iter_", None)
        loss = getattr(model, "loss_", None)
        loss_curve = getattr(model, "loss_curve_", [])
        diag.append(f"n_iter_={n_iter}, final_loss={loss}")
        diag.append(f"loss_curve_len={len(loss_curve)}")

    elif model_name in {"logistic_regression", "linear_svm"}:
        n_iter = getattr(model, "n_iter_", None)
        coef = getattr(model, "coef_", None)
        diag.append(f"n_iter_={n_iter}")
        if coef is not None:
            arr = np.asarray(coef)
            diag.append(f"coef_shape={arr.shape}")

    elif model_name == "knn":
        diag.append(
            f"n_neighbors={getattr(model, 'n_neighbors', None)}, metric={getattr(model, 'effective_metric_', 'n/a')}"
        )

    elif model_name == "naive_bayes":
        prior = getattr(model, "class_prior_", None)
        if prior is not None:
            arr = np.asarray(prior)
            diag.append("class_prior=" + ",".join(f"{p:.4f}" for p in arr))

    elif model_name == "lightgbm":
        best_iter = getattr(model, "best_iteration_", None)
        diag.append(f"best_iteration_={best_iter}")
        booster = getattr(model, "booster_", None)
        if booster is not None:
            try:
                diag.append(f"num_trees={booster.num_trees()}")
            except Exception:
                pass
            try:
                imp = booster.feature_importance(importance_type="gain")
                diag.append(f"top_features(gain): {top_features(feature_names, imp)}")
            except Exception:
                pass

    return diag


def prepare_data(cfg: CommonConfig) -> PreparedData:
    vlog(cfg.verbose, f"Loading parquet from: {cfg.input}")

    if not cfg.input.exists():
        raise FileNotFoundError(f"Input parquet not found: {cfg.input}")

    df = pd.read_parquet(cfg.input)
    vlog(cfg.verbose, f"Loaded dataframe with shape={df.shape}")

    if cfg.target not in df.columns:
        raise ValueError(f"Target column '{cfg.target}' not found in parquet")
    if cfg.count_col not in df.columns:
        raise ValueError(f"Count column '{cfg.count_col}' not found in parquet")

    if cfg.binary:
        y_raw = (df[cfg.target].to_numpy() <= cfg.threshold).astype(np.int32)
        mode_text = f"binary ({cfg.target} <= {cfg.threshold})"
    else:
        y_raw = df[cfg.target].to_numpy()
        mode_text = f"multiclass ({cfg.target})"
    vlog(cfg.verbose, f"Target mode: {mode_text}")

    weights_raw = pd.to_numeric(df[cfg.count_col], errors="coerce").to_numpy(dtype=float)

    explicit_metrics = parse_csv_list(cfg.metrics)
    if explicit_metrics:
        vlog(cfg.verbose, f"Using explicit feature list with {len(explicit_metrics)} columns")
        missing = [c for c in explicit_metrics if c not in df.columns]
        if missing:
            raise ValueError(f"Columns from --metrics not found: {missing}")
        feature_cols = [c for c in explicit_metrics if c != "npn_repr"]
        if len(feature_cols) != len(explicit_metrics):
            print("[warning] 'npn_repr' was removed and will not be used as a feature.")
    else:
        excluded = {cfg.target, cfg.count_col, "npn_repr"}
        feature_cols = [
            c
            for c in df.columns
            if c not in excluded and pd.api.types.is_numeric_dtype(df[c])
        ]
        vlog(cfg.verbose, f"Auto-selected {len(feature_cols)} numeric features")

    if not feature_cols:
        raise ValueError("No features available. Check --metrics or the parquet input.")

    x_df = df[feature_cols].apply(pd.to_numeric, errors="coerce")

    valid_mask = np.isfinite(x_df.to_numpy()).all(axis=1)
    valid_mask &= np.isfinite(weights_raw)
    if cfg.binary:
        valid_mask &= np.isfinite(y_raw)
    vlog(cfg.verbose, f"Rows before filtering: {len(df)}, valid rows: {int(valid_mask.sum())}")

    x = x_df.loc[valid_mask].to_numpy(dtype=float)
    y = np.asarray(y_raw)[valid_mask]
    w = weights_raw[valid_mask]

    if len(np.unique(y)) < 2:
        raise ValueError("Target has fewer than 2 classes after cleaning")
    if np.any(w < 0):
        raise ValueError("Weight column contains negative values")
    if np.all(w == 0):
        raise ValueError("All weights are 0")

    positive = w > 0
    x = x[positive]
    y = y[positive]
    w = w[positive]
    vlog(
        cfg.verbose,
        f"Rows after dropping non-positive weights: {len(y)}; weight sum={float(w.sum()):.2f}",
    )

    if cfg.max_rows is not None:
        if cfg.max_rows <= 0:
            raise ValueError("--max-rows must be > 0")
        if cfg.max_rows < len(y):
            vlog(cfg.verbose, f"Applying stratified subsample: max_rows={cfg.max_rows}")
            x, _, y, _, w, _ = train_test_split(
                x,
                y,
                w,
                train_size=cfg.max_rows,
                random_state=cfg.seed,
                stratify=y,
            )
            vlog(
                cfg.verbose,
                f"Rows after subsampling: {len(y)}; weight sum={float(w.sum()):.2f}",
            )

    unique_classes, class_counts = np.unique(y, return_counts=True)
    n_classes = int(unique_classes.shape[0])

    if cfg.verbose:
        class_summary = ", ".join(
            f"{int(cls)}:{int(cnt)}" for cls, cnt in zip(unique_classes, class_counts)
        )
        vlog(cfg.verbose, f"Class distribution (rows): {class_summary}")

    if n_classes > 1 and int(class_counts.min()) >= 2:
        stratify = y
    else:
        stratify = None
        if n_classes > 1:
            rare_classes = unique_classes[class_counts < 2]
            print(
                "[warning] Stratified split disabled because some classes have fewer than 2 "
                f"samples: {rare_classes.tolist()}"
            )

    try:
        x_train, x_test, y_train, y_test, w_train, w_test = train_test_split(
            x,
            y,
            w,
            test_size=cfg.test_size,
            random_state=cfg.seed,
            stratify=stratify,
        )
    except ValueError as exc:
        if stratify is not None:
            print(
                "[warning] Stratified split failed; falling back to non-stratified split. "
                f"Reason: {exc}"
            )
            x_train, x_test, y_train, y_test, w_train, w_test = train_test_split(
                x,
                y,
                w,
                test_size=cfg.test_size,
                random_state=cfg.seed,
                stratify=None,
            )
        else:
            raise

    vlog(
        cfg.verbose,
        "Split completed: "
        f"train_rows={len(y_train)}, test_rows={len(y_test)}, "
        f"train_weight_sum={float(w_train.sum()):.2f}, test_weight_sum={float(w_test.sum()):.2f}",
    )

    if cfg.normalize:
        # Normalize with train-set statistics only to avoid test leakage.
        means = np.mean(x_train, axis=0)
        stds = np.std(x_train, axis=0)
        stds_safe = np.where(stds == 0.0, 1.0, stds)
        x_train = (x_train - means) / stds_safe
        x_test = (x_test - means) / stds_safe
        vlog(cfg.verbose, "Applied z-score normalization using train-set statistics")

    return PreparedData(
        x_train=x_train,
        x_test=x_test,
        y_train=y_train,
        y_test=y_test,
        w_train=w_train,
        w_test=w_test,
        mode_text=mode_text,
        feature_cols=feature_cols,
        n_classes=n_classes,
        rows_used=len(y),
    )


def run_models(
    cfg: CommonConfig,
    data: PreparedData,
    builders: list[tuple[str, Callable[[], Any]]],
) -> list[RunResult]:
    print("\n==================== Training Configuration ====================")
    print(f"Input parquet      : {cfg.input}")
    print(f"Mode               : {data.mode_text}")
    print(f"Rows used          : {data.rows_used}")
    print(f"Train/Test         : {len(data.y_train)} / {len(data.y_test)}")
    print(f"Weight column      : {cfg.count_col}")
    print(f"Num classes        : {data.n_classes}")
    print(f"Num features       : {len(data.feature_cols)}")
    print(f"Features           : {', '.join(data.feature_cols)}")
    print(f"Models             : {', '.join(name for name, _ in builders)}")
    print(f"Train verbosity    : {cfg.train_verbose}")
    print(f"Normalize features : {cfg.normalize}")
    print("================================================================\n")

    results: list[RunResult] = []

    for model_name, builder in builders:
        start = time.perf_counter()
        try:
            vlog(cfg.verbose, f"Building model: {model_name}")
            model = builder()
            use_weight = supports_sample_weight(model)
            vlog(cfg.verbose, f"Model {model_name}: supports sample_weight={use_weight}")

            fit_kwargs: dict[str, Any] = {}
            if use_weight:
                fit_kwargs["sample_weight"] = data.w_train

            if model_name == "lightgbm" and cfg.train_verbose > 0:
                import lightgbm as lgb  # type: ignore

                fit_kwargs["eval_set"] = [(data.x_test, data.y_test)]
                fit_kwargs["eval_sample_weight"] = [data.w_test]
                fit_kwargs["callbacks"] = [
                    lgb.log_evaluation(period=max(1, cfg.lgb_log_period)),
                ]

            vlog(cfg.verbose, f"Training model: {model_name}")
            model.fit(data.x_train, data.y_train, **fit_kwargs)
            vlog(cfg.verbose, f"Training finished for: {model_name}")

            if cfg.verbose and hasattr(model, "n_iter_"):
                try:
                    vlog(cfg.verbose, f"Model {model_name} n_iter_: {getattr(model, 'n_iter_')}")
                except Exception:
                    pass

            y_pred = model.predict(data.x_test)
            scores = evaluate_predictions(
                y_true=data.y_test,
                y_pred=np.asarray(y_pred),
                sample_weight=data.w_test,
                is_binary=cfg.binary,
            )
            diagnostics = model_specific_diagnostics(model_name, model, data.feature_cols)

            elapsed = time.perf_counter() - start
            vlog(
                cfg.verbose,
                (
                    f"Model {model_name} done in {elapsed:.3f}s | "
                    f"acc_w={scores.get('accuracy_w', 0.0):.4f}, "
                    f"f1_macro_w={scores.get('f1_macro_w', 0.0):.4f}"
                ),
            )

            results.append(
                RunResult(
                    model=model_name,
                    ok=True,
                    train_weighted=use_weight,
                    fit_seconds=elapsed,
                    accuracy_w=scores.get("accuracy_w"),
                    f1_macro_w=scores.get("f1_macro_w"),
                    precision_macro_w=scores.get("precision_macro_w"),
                    recall_macro_w=scores.get("recall_macro_w"),
                    balanced_accuracy_w=scores.get("balanced_accuracy_w"),
                    f1_binary_w=scores.get("f1_binary_w"),
                    diagnostics=diagnostics,
                )
            )
        except Exception as exc:  # noqa: BLE001
            elapsed = time.perf_counter() - start
            vlog(cfg.verbose, f"Model {model_name} failed after {elapsed:.3f}s: {exc}")
            results.append(
                RunResult(
                    model=model_name,
                    ok=False,
                    train_weighted=False,
                    fit_seconds=elapsed,
                    error=str(exc),
                    diagnostics=[],
                )
            )

    return results


def print_and_save_results(cfg: CommonConfig, results: list[RunResult]) -> None:
    ok_results = [r for r in results if r.ok]
    bad_results = [r for r in results if not r.ok]
    ok_results.sort(key=lambda r: (r.f1_macro_w if r.f1_macro_w is not None else -1.0), reverse=True)

    headers = [
        "model",
        "status",
        "train_w",
        "acc_w",
        "f1_macro_w",
        "prec_macro_w",
        "rec_macro_w",
        "bal_acc_w",
    ]
    if cfg.binary:
        headers.append("f1_binary_w")
    headers.append("fit_s")

    rows: list[list[str]] = []
    for r in ok_results + bad_results:
        row = [
            r.model,
            "ok" if r.ok else "fail",
            "yes" if r.train_weighted else "no",
            float_or_dash(r.accuracy_w),
            float_or_dash(r.f1_macro_w),
            float_or_dash(r.precision_macro_w),
            float_or_dash(r.recall_macro_w),
            float_or_dash(r.balanced_accuracy_w),
        ]
        if cfg.binary:
            row.append(float_or_dash(r.f1_binary_w))
        row.append(float_or_dash(r.fit_seconds, ndigits=3))
        rows.append(row)

    print("========================= Model Results =========================")
    print(make_table(headers, rows))
    print("================================================================\n")

    if bad_results:
        print("Model errors:")
        for r in bad_results:
            print(f"- {r.model}: {r.error}")
        print()

    print("Model-specific diagnostics:")
    for r in ok_results:
        print(f"- {r.model}:")
        if r.diagnostics:
            for item in r.diagnostics:
                print(f"  - {item}")
        else:
            print("  - n/a")
    print()

    if cfg.output_csv is not None:
        out_records = []
        for r in ok_results + bad_results:
            out_records.append(
                {
                    "model": r.model,
                    "status": "ok" if r.ok else "fail",
                    "train_weighted": r.train_weighted,
                    "accuracy_w": r.accuracy_w,
                    "f1_macro_w": r.f1_macro_w,
                    "precision_macro_w": r.precision_macro_w,
                    "recall_macro_w": r.recall_macro_w,
                    "balanced_accuracy_w": r.balanced_accuracy_w,
                    "f1_binary_w": r.f1_binary_w,
                    "fit_seconds": r.fit_seconds,
                    "error": r.error,
                }
            )

        cfg.output_csv.parent.mkdir(parents=True, exist_ok=True)
        pd.DataFrame(out_records).to_csv(cfg.output_csv, index=False)
        print(f"Results written to: {cfg.output_csv}")


def run_single(cfg: CommonConfig, model_name: str, builder: Callable[[], Any]) -> None:
    data = prepare_data(cfg)
    results = run_models(cfg, data, [(model_name, builder)])
    print_and_save_results(cfg, results)


def run_pytorch_mlp(
    cfg: CommonConfig,
    data: PreparedData,
    hidden_layers: tuple[int, ...],
    epochs: int,
    batch_size: int,
    learning_rate: float,
    weight_decay: float,
    optimizer_name: str,
    max_lr: Optional[float],
    dropout: float,
    activation: str,
    batch_norm: bool,
    hidden_dropouts: Optional[tuple[float, ...]],
    device: str,
    deterministic: bool,
) -> RunResult:
    try:
        import torch
        import torch.nn as nn
        import torch.nn.functional as f
    except Exception as exc:  # noqa: BLE001
        raise RuntimeError("torch is not installed. Use uv run --with torch ...") from exc

    if optimizer_name not in {"adam", "adamw", "sgd"}:
        raise typer.BadParameter("--optimizer must be 'adam', 'adamw' or 'sgd'")
    if activation not in {"relu", "gelu"}:
        raise typer.BadParameter("--activation must be 'relu' or 'gelu'")

    if batch_size <= 0:
        raise typer.BadParameter("--batch-size must be > 0")
    if epochs <= 0:
        raise typer.BadParameter("--epochs must be > 0")
    if learning_rate <= 0:
        raise typer.BadParameter("--learning-rate must be > 0")
    if weight_decay < 0:
        raise typer.BadParameter("--weight-decay must be >= 0")
    if not (0.0 <= dropout < 1.0):
        raise typer.BadParameter("--dropout must be in [0, 1)")
    if hidden_dropouts is not None:
        if len(hidden_dropouts) != len(hidden_layers):
            raise typer.BadParameter("--hidden-dropouts must have one value per hidden layer")
        if any((d < 0.0 or d >= 1.0) for d in hidden_dropouts):
            raise typer.BadParameter("--hidden-dropouts values must be in [0, 1)")
    if max_lr is not None and max_lr <= 0:
        raise typer.BadParameter("--max-lr must be > 0")

    random.seed(cfg.seed)
    np.random.seed(cfg.seed)
    torch.manual_seed(cfg.seed)
    if torch.cuda.is_available():
        torch.cuda.manual_seed_all(cfg.seed)

    if deterministic:
        try:
            torch.use_deterministic_algorithms(True)
        except Exception:
            pass
        if hasattr(torch.backends, "cudnn"):
            torch.backends.cudnn.deterministic = True
            torch.backends.cudnn.benchmark = False

    if device == "auto":
        chosen_device = "cuda" if torch.cuda.is_available() else "cpu"
    else:
        chosen_device = device
    if chosen_device == "cuda" and not torch.cuda.is_available():
        raise RuntimeError("CUDA requested but not available")

    dev = torch.device(chosen_device)

    x_train = torch.tensor(data.x_train, dtype=torch.float32)
    y_train = torch.tensor(data.y_train, dtype=torch.long)
    w_train = torch.tensor(data.w_train, dtype=torch.float32)
    x_test = torch.tensor(data.x_test, dtype=torch.float32)
    n_rows = x_train.shape[0]

    input_dim = int(x_train.shape[1])
    n_classes = int(data.n_classes)

    layers: list[nn.Module] = []
    prev = input_dim
    for i, h in enumerate(hidden_layers):
        layers.append(nn.Linear(prev, h))
        if batch_norm:
            layers.append(nn.BatchNorm1d(h))
        if activation == "gelu":
            layers.append(nn.GELU())
        else:
            layers.append(nn.ReLU())
        drop_p = hidden_dropouts[i] if hidden_dropouts is not None else dropout
        if drop_p > 0.0:
            layers.append(nn.Dropout(p=drop_p))
        prev = h
    layers.append(nn.Linear(prev, n_classes))
    model = nn.Sequential(*layers).to(dev)

    if optimizer_name == "adam":
        optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate, weight_decay=weight_decay)
    elif optimizer_name == "adamw":
        optimizer = torch.optim.AdamW(model.parameters(), lr=learning_rate, weight_decay=weight_decay)
    else:
        optimizer = torch.optim.SGD(model.parameters(), lr=learning_rate, weight_decay=weight_decay, momentum=0.9)

    scheduler = None
    if max_lr is not None:
        steps_per_epoch = max(1, int(np.ceil(n_rows / batch_size)))
        scheduler = torch.optim.lr_scheduler.OneCycleLR(
            optimizer,
            max_lr=max_lr,
            epochs=epochs,
            steps_per_epoch=steps_per_epoch,
        )

    start = time.perf_counter()
    model.train()
    epoch_losses: list[float] = []

    for epoch in range(1, epochs + 1):
        perm = torch.randperm(n_rows)
        total_loss = 0.0
        total_weight = 0.0

        for i in range(0, n_rows, batch_size):
            idx = perm[i : i + batch_size]
            xb = x_train[idx].to(dev)
            yb = y_train[idx].to(dev)
            wb = w_train[idx].to(dev)

            optimizer.zero_grad(set_to_none=True)
            logits = model(xb)
            per_sample_loss = f.cross_entropy(logits, yb, reduction="none")
            batch_weight_sum = wb.sum().clamp_min(1e-12)
            loss = (per_sample_loss * wb).sum() / batch_weight_sum
            loss.backward()
            optimizer.step()
            if scheduler is not None:
                scheduler.step()

            total_loss += float((per_sample_loss * wb).sum().detach().cpu().item())
            total_weight += float(batch_weight_sum.detach().cpu().item())

        mean_loss = total_loss / max(total_weight, 1e-12)
        epoch_losses.append(mean_loss)
        if cfg.train_verbose > 0:
            print(f"[train][pytorch_mlp] epoch={epoch}/{epochs} loss={mean_loss:.6f}")

    model.eval()
    with torch.no_grad():
        logits = model(x_test.to(dev))
        y_pred = torch.argmax(logits, dim=1).cpu().numpy()

    elapsed = time.perf_counter() - start
    scores = evaluate_predictions(
        y_true=data.y_test,
        y_pred=np.asarray(y_pred),
        sample_weight=data.w_test,
        is_binary=cfg.binary,
    )

    diagnostics = [
        f"framework=pytorch",
        f"device={chosen_device}",
        f"epochs={epochs}, batch_size={batch_size}",
        f"optimizer={optimizer_name}, lr={learning_rate}, weight_decay={weight_decay}, max_lr={max_lr}",
        f"hidden_layers={hidden_layers}, activation={activation}, batch_norm={batch_norm}",
        f"dropout={dropout}, hidden_dropouts={hidden_dropouts}",
        f"final_epoch_loss={epoch_losses[-1]:.6f}",
        f"best_epoch_loss={min(epoch_losses):.6f}",
    ]

    return RunResult(
        model="pytorch_mlp",
        ok=True,
        train_weighted=True,
        fit_seconds=elapsed,
        accuracy_w=scores.get("accuracy_w"),
        f1_macro_w=scores.get("f1_macro_w"),
        precision_macro_w=scores.get("precision_macro_w"),
        recall_macro_w=scores.get("recall_macro_w"),
        balanced_accuracy_w=scores.get("balanced_accuracy_w"),
        f1_binary_w=scores.get("f1_binary_w"),
        diagnostics=diagnostics,
    )


@app.callback()
def main(
    ctx: typer.Context,
    input: Path = typer.Option(Path("results/dataset.parquet"), "--input", help="Input parquet file"),
    target: str = typer.Option("min_gates", "--target", help="Target column"),
    count_col: str = typer.Option("count", "--count-col", help="Weight/frequency column"),
    binary: bool = typer.Option(False, "--binary", help="Use binary target: target <= threshold"),
    threshold: float = typer.Option(9.0, "--threshold", help="Binary threshold"),
    metrics: Optional[str] = typer.Option(None, "--metrics", help="CSV feature list. npn_repr is excluded."),
    test_size: float = typer.Option(0.2, "--test-size", help="Test split ratio"),
    seed: int = typer.Option(42, "--seed", help="Random seed"),
    output_csv: Optional[Path] = typer.Option(None, "--output-csv", help="Write results CSV"),
    verbose: bool = typer.Option(False, "--verbose", help="Verbose pipeline logs"),
    train_verbose: int = typer.Option(
        0,
        "--train-verbose",
        help="Internal model training verbosity (epochs/iterations where supported)",
    ),
    lgb_log_period: int = typer.Option(
        50,
        "--lgb-log-period",
        help="LightGBM log period in boosting rounds when --train-verbose > 0",
    ),
    normalize: bool = typer.Option(False, "--normalize", help="Apply z-score normalization to features"),
    max_rows: Optional[int] = typer.Option(None, "--max-rows", help="Optional stratified row cap before train/test split"),
) -> None:
    """Model training CLI."""

    if not (0.0 < test_size < 1.0):
        raise typer.BadParameter("--test-size must be between 0 and 1")
    if train_verbose < 0:
        raise typer.BadParameter("--train-verbose must be >= 0")
    if lgb_log_period <= 0:
        raise typer.BadParameter("--lgb-log-period must be > 0")
    if max_rows is not None and max_rows <= 0:
        raise typer.BadParameter("--max-rows must be > 0")

    ctx.obj = CommonConfig(
        input=input,
        target=target,
        count_col=count_col,
        binary=binary,
        threshold=threshold,
        metrics=metrics,
        test_size=test_size,
        seed=seed,
        output_csv=output_csv,
        verbose=verbose,
        train_verbose=train_verbose,
        lgb_log_period=lgb_log_period,
        normalize=normalize,
        max_rows=max_rows,
    )


def cfg_from_ctx(ctx: typer.Context) -> CommonConfig:
    obj = ctx.obj
    if obj is None:
        raise RuntimeError("Common options were not initialized")
    return obj


@app.command("compare")
def compare(
    ctx: typer.Context,
    models: str = typer.Option("all", "--models", help="CSV model list or 'all'"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    requested = parse_csv_list(models)
    if models.strip().lower() == "all" or not requested:
        requested = DEFAULT_MODELS.copy()

    unknown = sorted(set(requested) - set(DEFAULT_MODELS))
    if unknown:
        raise typer.BadParameter(f"Unsupported models: {unknown}")

    data = prepare_data(cfg)
    builders: list[tuple[str, Callable[[], Any]]] = [
        (name, lambda n=name: make_default_model(n, cfg.seed, data.n_classes, cfg.train_verbose))
        for name in requested
    ]
    results = run_models(cfg, data, builders)
    print_and_save_results(cfg, results)


@app.command("mlp")
def cmd_mlp(
    ctx: typer.Context,
    hidden_layers: str = typer.Option("64,32", "--hidden-layers", help="CSV hidden sizes, e.g. 64,32"),
    max_iter: int = typer.Option(400, "--max-iter", help="Maximum training iterations"),
    alpha: float = typer.Option(0.0001, "--alpha", help="L2 regularization strength"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    layers = parse_hidden_layers(hidden_layers)
    run_single(
        cfg,
        "mlp",
        lambda: MLPClassifier(
            hidden_layer_sizes=layers,
            max_iter=max_iter,
            alpha=alpha,
            random_state=cfg.seed,
            verbose=(cfg.train_verbose > 0),
        ),
    )


@app.command("decision-tree")
def cmd_decision_tree(
    ctx: typer.Context,
    max_depth: Optional[int] = typer.Option(None, "--max-depth", help="Maximum tree depth"),
    min_samples_split: int = typer.Option(2, "--min-samples-split", help="Minimum samples to split"),
    min_samples_leaf: int = typer.Option(1, "--min-samples-leaf", help="Minimum samples per leaf"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    run_single(
        cfg,
        "decision_tree",
        lambda: DecisionTreeClassifier(
            random_state=cfg.seed,
            max_depth=max_depth,
            min_samples_split=min_samples_split,
            min_samples_leaf=min_samples_leaf,
        ),
    )


@app.command("random-forest")
def cmd_random_forest(
    ctx: typer.Context,
    n_estimators: int = typer.Option(300, "--n-estimators", help="Number of trees"),
    max_depth: Optional[int] = typer.Option(None, "--max-depth", help="Maximum tree depth"),
    min_samples_split: int = typer.Option(2, "--min-samples-split", help="Minimum samples to split"),
    min_samples_leaf: int = typer.Option(1, "--min-samples-leaf", help="Minimum samples per leaf"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    run_single(
        cfg,
        "random_forest",
        lambda: RandomForestClassifier(
            n_estimators=n_estimators,
            random_state=cfg.seed,
            n_jobs=-1,
            max_depth=max_depth,
            min_samples_split=min_samples_split,
            min_samples_leaf=min_samples_leaf,
            verbose=cfg.train_verbose,
        ),
    )


@app.command("extra-trees")
def cmd_extra_trees(
    ctx: typer.Context,
    n_estimators: int = typer.Option(300, "--n-estimators", help="Number of trees"),
    max_depth: Optional[int] = typer.Option(None, "--max-depth", help="Maximum tree depth"),
    min_samples_split: int = typer.Option(2, "--min-samples-split", help="Minimum samples to split"),
    min_samples_leaf: int = typer.Option(1, "--min-samples-leaf", help="Minimum samples per leaf"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    run_single(
        cfg,
        "extra_trees",
        lambda: ExtraTreesClassifier(
            n_estimators=n_estimators,
            random_state=cfg.seed,
            n_jobs=-1,
            max_depth=max_depth,
            min_samples_split=min_samples_split,
            min_samples_leaf=min_samples_leaf,
            verbose=cfg.train_verbose,
        ),
    )


@app.command("gradient-boosting")
def cmd_gradient_boosting(
    ctx: typer.Context,
    n_estimators: int = typer.Option(200, "--n-estimators", help="Number of boosting stages"),
    learning_rate: float = typer.Option(0.05, "--learning-rate", help="Learning rate"),
    max_depth: int = typer.Option(3, "--max-depth", help="Base tree max depth"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    run_single(
        cfg,
        "gradient_boosting",
        lambda: GradientBoostingClassifier(
            random_state=cfg.seed,
            n_estimators=n_estimators,
            learning_rate=learning_rate,
            max_depth=max_depth,
            verbose=cfg.train_verbose,
        ),
    )


@app.command("logistic-regression")
def cmd_logistic_regression(
    ctx: typer.Context,
    max_iter: int = typer.Option(2000, "--max-iter", help="Maximum optimization iterations"),
    c: float = typer.Option(1.0, "--c", help="Inverse regularization strength"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    data = prepare_data(cfg)

    if data.n_classes > 2:
        builder = lambda: LogisticRegression(
            max_iter=max_iter,
            random_state=cfg.seed,
            C=c,
            multi_class="multinomial",
            verbose=cfg.train_verbose,
        )
    else:
        builder = lambda: LogisticRegression(
            max_iter=max_iter,
            random_state=cfg.seed,
            C=c,
            verbose=cfg.train_verbose,
        )

    results = run_models(cfg, data, [("logistic_regression", builder)])
    print_and_save_results(cfg, results)


@app.command("linear-svm")
def cmd_linear_svm(
    ctx: typer.Context,
    c: float = typer.Option(1.0, "--c", help="Regularization parameter"),
    max_iter: int = typer.Option(3000, "--max-iter", help="Maximum optimization iterations"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    run_single(
        cfg,
        "linear_svm",
        lambda: LinearSVC(
            random_state=cfg.seed,
            C=c,
            max_iter=max_iter,
            verbose=cfg.train_verbose,
        ),
    )


@app.command("knn")
def cmd_knn(
    ctx: typer.Context,
    n_neighbors: int = typer.Option(25, "--n-neighbors", help="Number of neighbors"),
    weights: str = typer.Option("uniform", "--weights", help="uniform or distance"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    if weights not in {"uniform", "distance"}:
        raise typer.BadParameter("--weights must be 'uniform' or 'distance'")
    run_single(
        cfg,
        "knn",
        lambda: KNeighborsClassifier(n_neighbors=n_neighbors, weights=weights),
    )


@app.command("naive-bayes")
def cmd_naive_bayes(
    ctx: typer.Context,
    var_smoothing: float = typer.Option(1e-9, "--var-smoothing", help="Variance smoothing"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    run_single(
        cfg,
        "naive_bayes",
        lambda: GaussianNB(var_smoothing=var_smoothing),
    )


@app.command("lightgbm")
def cmd_lightgbm(
    ctx: typer.Context,
    n_estimators: int = typer.Option(500, "--n-estimators", help="Number of boosting rounds"),
    learning_rate: float = typer.Option(0.05, "--learning-rate", help="Learning rate"),
    num_leaves: int = typer.Option(31, "--num-leaves", help="Number of leaves"),
    subsample: float = typer.Option(0.9, "--subsample", help="Row subsampling ratio"),
    colsample_bytree: float = typer.Option(0.9, "--colsample-bytree", help="Feature subsampling ratio"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    data = prepare_data(cfg)

    try:
        import lightgbm as lgb  # type: ignore
    except Exception as exc:  # noqa: BLE001
        raise RuntimeError("lightgbm is not installed. Use uv run --with lightgbm ...") from exc

    objective = "binary" if data.n_classes == 2 else "multiclass"
    params = {
        "objective": objective,
        "n_estimators": n_estimators,
        "learning_rate": learning_rate,
        "num_leaves": num_leaves,
        "subsample": subsample,
        "colsample_bytree": colsample_bytree,
        "random_state": cfg.seed,
        "n_jobs": -1,
        "verbose": (1 if cfg.train_verbose > 0 else -1),
    }
    if data.n_classes > 2:
        params["num_class"] = data.n_classes

    def _builder() -> Any:
        return lgb.LGBMClassifier(**params)

    results = run_models(cfg, data, [("lightgbm", _builder)])
    print_and_save_results(cfg, results)


@app.command("pytorch-mlp")
def cmd_pytorch_mlp(
    ctx: typer.Context,
    hidden_layers: str = typer.Option("128,64", "--hidden-layers", help="CSV hidden sizes, e.g. 128,64"),
    epochs: int = typer.Option(100, "--epochs", help="Training epochs"),
    batch_size: int = typer.Option(256, "--batch-size", help="Mini-batch size"),
    learning_rate: float = typer.Option(0.001, "--learning-rate", help="Learning rate"),
    weight_decay: float = typer.Option(0.0, "--weight-decay", help="L2 weight decay"),
    optimizer: str = typer.Option("adam", "--optimizer", help="adam, adamw or sgd"),
    max_lr: Optional[float] = typer.Option(None, "--max-lr", help="Optional OneCycleLR max learning rate"),
    dropout: float = typer.Option(0.0, "--dropout", help="Dropout probability"),
    activation: str = typer.Option("relu", "--activation", help="relu or gelu"),
    batch_norm: bool = typer.Option(False, "--batch-norm/--no-batch-norm", help="Enable BatchNorm1d after hidden linear layers"),
    hidden_dropouts: Optional[str] = typer.Option(None, "--hidden-dropouts", help="CSV dropout per hidden layer, e.g. 0.3,0.3,0.2,0.0"),
    device: str = typer.Option("auto", "--device", help="auto, cpu, or cuda"),
    deterministic: bool = typer.Option(True, "--deterministic/--non-deterministic", help="Deterministic training mode"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    layers = parse_hidden_layers(hidden_layers)
    if device not in {"auto", "cpu", "cuda"}:
        raise typer.BadParameter("--device must be one of: auto, cpu, cuda")
    parsed_hidden_dropouts = parse_float_csv(hidden_dropouts) if hidden_dropouts else None

    data = prepare_data(cfg)
    print("\n==================== Training Configuration ====================")
    print(f"Input parquet      : {cfg.input}")
    print(f"Mode               : {data.mode_text}")
    print(f"Rows used          : {data.rows_used}")
    print(f"Train/Test         : {len(data.y_train)} / {len(data.y_test)}")
    print(f"Weight column      : {cfg.count_col}")
    print(f"Num classes        : {data.n_classes}")
    print(f"Num features       : {len(data.feature_cols)}")
    print(f"Features           : {', '.join(data.feature_cols)}")
    print("Models             : pytorch_mlp")
    print(f"Train verbosity    : {cfg.train_verbose}")
    print(f"Normalize features : {cfg.normalize}")
    print("================================================================\n")

    try:
        result = run_pytorch_mlp(
            cfg=cfg,
            data=data,
            hidden_layers=layers,
            epochs=epochs,
            batch_size=batch_size,
            learning_rate=learning_rate,
            weight_decay=weight_decay,
            optimizer_name=optimizer,
            max_lr=max_lr,
            dropout=dropout,
            activation=activation,
            batch_norm=batch_norm,
            hidden_dropouts=parsed_hidden_dropouts,
            device=device,
            deterministic=deterministic,
        )
        results = [result]
    except Exception as exc:  # noqa: BLE001
        results = [
            RunResult(
                model="pytorch_mlp",
                ok=False,
                train_weighted=False,
                fit_seconds=0.0,
                error=str(exc),
                diagnostics=[],
            )
        ]

    print_and_save_results(cfg, results)


@app.command("pytorch-mlp-smc2026")
def cmd_pytorch_mlp_smc2026(
    ctx: typer.Context,
    hidden_layers: str = typer.Option(
        "256,512,256,128",
        "--hidden-layers",
        help="CSV hidden sizes (paper default: 256,512,256,128)",
    ),
    epochs: int = typer.Option(15, "--epochs", help="Training epochs (paper default: 15)"),
    batch_size: int = typer.Option(4096, "--batch-size", help="Mini-batch size (paper default: 4096)"),
    learning_rate: float = typer.Option(0.001, "--learning-rate", help="Base learning rate (paper default: 1e-3)"),
    max_lr: float = typer.Option(0.01, "--max-lr", help="OneCycle max learning rate (paper default: 1e-2)"),
    weight_decay: float = typer.Option(0.0001, "--weight-decay", help="Weight decay (paper default: 1e-4)"),
    hidden_dropouts: str = typer.Option("0.3,0.3,0.2,0.0", "--hidden-dropouts", help="Dropout per hidden layer"),
    device: str = typer.Option("auto", "--device", help="auto, cpu, or cuda"),
    deterministic: bool = typer.Option(True, "--deterministic/--non-deterministic", help="Deterministic training mode"),
) -> None:
    cfg = cfg_from_ctx(ctx)
    layers = parse_hidden_layers(hidden_layers)
    parsed_hidden_dropouts = parse_float_csv(hidden_dropouts)
    if device not in {"auto", "cpu", "cuda"}:
        raise typer.BadParameter("--device must be one of: auto, cpu, cuda")

    data = prepare_data(cfg)
    print("\n==================== Training Configuration ====================")
    print(f"Input parquet      : {cfg.input}")
    print(f"Mode               : {data.mode_text}")
    print(f"Rows used          : {data.rows_used}")
    print(f"Train/Test         : {len(data.y_train)} / {len(data.y_test)}")
    print(f"Weight column      : {cfg.count_col}")
    print(f"Num classes        : {data.n_classes}")
    print(f"Num features       : {len(data.feature_cols)}")
    print(f"Features           : {', '.join(data.feature_cols)}")
    print("Models             : pytorch_mlp_smc2026")
    print(f"Train verbosity    : {cfg.train_verbose}")
    print(f"Normalize features : {cfg.normalize}")
    print("================================================================\n")

    try:
        result = run_pytorch_mlp(
            cfg=cfg,
            data=data,
            hidden_layers=layers,
            epochs=epochs,
            batch_size=batch_size,
            learning_rate=learning_rate,
            weight_decay=weight_decay,
            optimizer_name="adamw",
            max_lr=max_lr,
            dropout=0.0,
            activation="gelu",
            batch_norm=True,
            hidden_dropouts=parsed_hidden_dropouts,
            device=device,
            deterministic=deterministic,
        )
        result.model = "pytorch_mlp_smc2026"
        results = [result]
    except Exception as exc:  # noqa: BLE001
        results = [
            RunResult(
                model="pytorch_mlp_smc2026",
                ok=False,
                train_weighted=False,
                fit_seconds=0.0,
                error=str(exc),
                diagnostics=[],
            )
        ]

    print_and_save_results(cfg, results)


if __name__ == "__main__":
    app()
