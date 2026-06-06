use async_trait::async_trait;

use super::types::{
    ConnectionTest, ExchangeError, ExchangeFundingEvent, ExchangeHolding, ExchangeSyncState,
    ExchangeTrade, ExchangeTransfer,
};

#[async_trait]
pub trait ExchangeConnector: Send + Sync {
    fn exchange_id(&self) -> &'static str;

    async fn test_connection(&self) -> Result<ConnectionTest, ExchangeError>;

    async fn sync_balances(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError>;

    async fn sync_positions(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeHolding>, ExchangeError>;

    async fn sync_trades(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeTrade>, ExchangeError>;

    async fn sync_transfers(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeTransfer>, ExchangeError>;

    async fn sync_funding(
        &self,
        state: &mut ExchangeSyncState,
    ) -> Result<Vec<ExchangeFundingEvent>, ExchangeError>;
}
