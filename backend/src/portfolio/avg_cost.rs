use crate::exchanges::types::ExchangeTrade;

pub fn average_cost_from_trades(trades: &[ExchangeTrade], asset: &str) -> Option<f64> {
    let mut qty = 0.0f64;
    let mut cost = 0.0f64;

    for t in trades {
        let base = t.symbol.trim_end_matches("USDT").trim_end_matches("USDC");
        if base != asset && t.symbol != asset {
            continue;
        }
        let trade_qty = t.quantity;
        let trade_cost = trade_qty * t.price;
        match t.side.as_str() {
            "buy" => {
                cost += trade_cost;
                qty += trade_qty;
            }
            "sell" => {
                if qty > 0.0 {
                    let avg = cost / qty;
                    qty -= trade_qty.min(qty);
                    cost = avg * qty;
                }
            }
            _ => {}
        }
    }

    if qty > 0.0 {
        Some(cost / qty)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn trade(side: &str, qty: f64, price: f64) -> ExchangeTrade {
        ExchangeTrade {
            external_id: "1".into(),
            symbol: "BTCUSDT".into(),
            side: side.into(),
            quantity: qty,
            price,
            fee: None,
            fee_asset: None,
            realized_pnl: None,
            executed_at: Utc::now(),
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn avg_cost_from_buys() {
        let trades = vec![trade("buy", 1.0, 100.0), trade("buy", 1.0, 200.0)];
        let avg = average_cost_from_trades(&trades, "BTC").unwrap();
        assert!((avg - 150.0).abs() < 0.01);
    }
}
