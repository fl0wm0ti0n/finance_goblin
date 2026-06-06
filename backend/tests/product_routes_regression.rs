//! US-0011 AC-4: ECharts product pages and analytics routes remain in frontend sources.

use std::fs;

const APP_TSX: &str = include_str!("../../frontend/src/App.tsx");
const WEALTH_TSX: &str = include_str!("../../frontend/src/pages/WealthPage.tsx");
const ANALYTICS_TSX: &str = include_str!("../../frontend/src/pages/AnalyticsEmbedPage.tsx");

#[test]
fn product_routes_registered_in_app() {
    for route in [
        "/forecast",
        "/wealth",
        "/planning",
        "/subscriptions",
        "/alerts",
    ] {
        assert!(
            APP_TSX.contains(&format!("path=\"{route}\"")),
            "missing route {route}"
        );
    }
}

#[test]
fn analytics_routes_registered() {
    for slug in [
        "platform-health",
        "cashflow",
        "subscriptions",
        "budgets",
        "portfolio",
        "forecast-horizons",
    ] {
        assert!(ANALYTICS_TSX.contains(slug), "missing analytics slug {slug}");
    }
    assert!(APP_TSX.contains("path=\"/analytics/:slug\""));
}

#[test]
fn wealth_uses_in_app_portfolio_analytics() {
    assert!(WEALTH_TSX.contains("to=\"/analytics/portfolio\""));
    assert!(!WEALTH_TSX.contains("VITE_GRAFANA_URL"));
    assert!(!WEALTH_TSX.contains("target=\"_blank\""));
}

#[test]
fn echarts_pages_still_import_chart_surfaces() {
    let root = format!("{}/../frontend/src/pages", env!("CARGO_MANIFEST_DIR"));
    let forecast = fs::read_to_string(format!("{root}/ForecastPage.tsx")).expect("forecast");
    let wealth = fs::read_to_string(format!("{root}/WealthPage.tsx")).expect("wealth");
    let planning = fs::read_to_string(format!("{root}/PlanningPage.tsx")).expect("planning");
    let subscriptions =
        fs::read_to_string(format!("{root}/SubscriptionsPage.tsx")).expect("subscriptions");

    assert!(forecast.contains("echarts") || forecast.contains("ECharts") || forecast.contains("Chart"));
    assert!(wealth.contains("WealthChart") || wealth.contains("echarts"));
    assert!(planning.contains("echarts") || planning.contains("Chart") || planning.contains("ECharts"));
    assert!(
        subscriptions.contains("echarts")
            || subscriptions.contains("Chart")
            || subscriptions.contains("ECharts")
    );
}
