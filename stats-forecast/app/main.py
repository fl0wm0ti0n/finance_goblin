"""StatsForecast sidecar — US-0009 (DEC-0049, DEC-0051)."""

from __future__ import annotations

import math
from datetime import datetime
from typing import Any

import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field
from statsforecast import StatsForecast
from statsforecast.models import AutoETS, MSTL, SeasonalNaive

app = FastAPI(title="stats-forecast", version="1.0.0")

MIN_POINTS = 12
MSTL_MIN_MONTHS = 24
MSTL_SEASONAL_STRENGTH = 0.35
WMAPE_LOW_CONFIDENCE = 0.35


class Point(BaseModel):
    ds: str
    y: float


class ForecastRequest(BaseModel):
    series_id: str
    freq: str = "MS"
    points: list[Point]
    horizon: int = Field(default=24, ge=1, le=48)
    level: list[int] = Field(default_factory=lambda: [90])
    model: str = "auto"


class ForecastPoint(BaseModel):
    ds: str
    y: float
    y_lo: float
    y_hi: float


class ForecastResponse(BaseModel):
    model_family: str
    seasonal_periods: list[int]
    seasonal_strength: float
    seasonal_detected: bool
    forecasts: list[ForecastPoint]
    backtest_wmape: float | None
    low_confidence: bool


def _parse_ds(ds: str) -> pd.Timestamp:
    return pd.to_datetime(ds)


def _seasonal_strength(y: np.ndarray, period: int = 12) -> float:
    if len(y) < period * 2:
        return 0.0
    try:
        fft = np.fft.rfft(y - np.mean(y))
        power = np.abs(fft) ** 2
        if len(power) <= 1:
            return 0.0
        idx = min(period, len(power) - 1)
        return float(power[idx] / (power.sum() + 1e-9))
    except Exception:
        return 0.0


def _wmape(actual: np.ndarray, predicted: np.ndarray) -> float:
    denom = np.sum(np.abs(actual))
    if denom < 1e-9:
        return 0.0
    return float(np.sum(np.abs(actual - predicted)) / denom)


def _select_model(n: int, strength: float, freq: str) -> tuple[Any, str, list[int]]:
    period = 12 if freq in ("MS", "M") else (52 if freq == "W" else 12)
    periods = [period]

    if n < MIN_POINTS:
        raise HTTPException(status_code=422, detail="insufficient_history")

    if n >= MSTL_MIN_MONTHS and strength >= MSTL_SEASONAL_STRENGTH:
        return (
            MSTL(season_length=period, trend_forecaster=AutoETS(season_length=period)),
            "MSTL+AutoETS",
            periods,
        )
    if n >= MIN_POINTS:
        return AutoETS(season_length=period), "AutoETS", periods
    return SeasonalNaive(season_length=period), "SeasonalNaive", periods


def _run_forecast(req: ForecastRequest) -> ForecastResponse:
    if len(req.points) < MIN_POINTS:
        raise HTTPException(status_code=422, detail="insufficient_history")

    df = pd.DataFrame(
        {"ds": [_parse_ds(p.ds) for p in req.points], "y": [p.y for p in req.points]}
    )
    df = df.sort_values("ds").drop_duplicates(subset=["ds"], keep="last")
    y = df["y"].to_numpy(dtype=float)
    strength = _seasonal_strength(y, 12 if req.freq in ("MS", "M") else 52)
    seasonal_detected = strength >= MSTL_SEASONAL_STRENGTH

    model, family, periods = _select_model(len(df), strength, req.freq)
    low_confidence = False
    backtest_wmape: float | None = None

    try:
        sf = StatsForecast(models=[model], freq=req.freq, n_jobs=1)
        sf.fit(df)
        level = req.level[0] if req.level else 90
        pred = sf.predict(h=req.horizon, level=[level])
    except Exception:
        family = "SeasonalNaive"
        periods = [12 if req.freq in ("MS", "M") else 52]
        sf = StatsForecast(
            models=[SeasonalNaive(season_length=periods[0])], freq=req.freq, n_jobs=1
        )
        sf.fit(df)
        level = req.level[0] if req.level else 90
        pred = sf.predict(h=req.horizon, level=[level])
        low_confidence = True

    if len(df) >= MSTL_MIN_MONTHS:
        try:
            cv = sf.cross_validation(h=6, n_windows=min(3, len(df) // 12), step_size=6)
            col = [c for c in cv.columns if c not in ("ds", "unique_id") and "-lo-" not in c and "-hi-" not in c]
            if col:
                actual = cv["y"].to_numpy()
                predicted = cv[col[0]].to_numpy()
                backtest_wmape = _wmape(actual, predicted)
                if backtest_wmape is not None and backtest_wmape > WMAPE_LOW_CONFIDENCE:
                    low_confidence = True
        except Exception:
            pass

    lo_col = f"{family}-lo-{level}" if f"{family}-lo-{level}" in pred.columns else None
    hi_col = f"{family}-hi-{level}" if f"{family}-hi-{level}" in pred.columns else None
    y_col = family if family in pred.columns else pred.columns[-1]

    forecasts: list[ForecastPoint] = []
    for _, row in pred.iterrows():
        y_val = float(row[y_col])
        y_lo = float(row[lo_col]) if lo_col else y_val * 0.85
        y_hi = float(row[hi_col]) if hi_col else y_val * 1.15
        forecasts.append(
            ForecastPoint(
                ds=row["ds"].strftime("%Y-%m-%d"),
                y=y_val,
                y_lo=y_lo,
                y_hi=y_hi,
            )
        )

    return ForecastResponse(
        model_family=family,
        seasonal_periods=periods,
        seasonal_strength=round(strength, 4),
        seasonal_detected=seasonal_detected,
        forecasts=forecasts,
        backtest_wmape=round(backtest_wmape, 4) if backtest_wmape is not None else None,
        low_confidence=low_confidence,
    )


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.post("/v1/forecast", response_model=ForecastResponse)
def forecast(req: ForecastRequest) -> ForecastResponse:
    return _run_forecast(req)
