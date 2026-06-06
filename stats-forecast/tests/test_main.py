"""Sidecar unit tests — health + sample forecast."""

from datetime import datetime, timedelta

import pytest
from fastapi.testclient import TestClient

from app.main import app

client = TestClient(app)


def test_health():
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"


def test_forecast_monthly_series():
    base = datetime(2023, 1, 1)
    points = []
    for i in range(18):
        ds = (base + timedelta(days=30 * i)).strftime("%Y-%m-%d")
        seasonal = 200 * (1 if i % 12 < 6 else -1)
        points.append({"ds": ds, "y": 1200.0 + seasonal + i * 10})

    resp = client.post(
        "/v1/forecast",
        json={
            "series_id": "household",
            "freq": "MS",
            "points": points,
            "horizon": 6,
            "level": [90],
            "model": "auto",
        },
    )
    assert resp.status_code == 200
    body = resp.json()
    assert body["model_family"] in ("AutoETS", "MSTL+AutoETS", "SeasonalNaive")
    assert len(body["forecasts"]) == 6
    for fc in body["forecasts"]:
        assert "y_lo" in fc and "y_hi" in fc
        assert fc["y_lo"] <= fc["y_hi"]


def test_insufficient_history():
    resp = client.post(
        "/v1/forecast",
        json={
            "series_id": "x",
            "freq": "MS",
            "points": [{"ds": "2024-01-01", "y": 100.0}],
            "horizon": 3,
            "level": [90],
        },
    )
    assert resp.status_code == 422
