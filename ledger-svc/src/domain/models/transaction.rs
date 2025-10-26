use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceStatus {
    Pending,
    Priced,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetId {
    pub symbol: String,
    pub kind: AssetKind,
    // pub chain: Option<String>,
    // pub contract: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Fiat,
    Crypto,
    NFT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivativeKind {
    Perpetual,
    Futures,
    Option,
    Leveraged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxKind {
    Spot,
    Swap,
    DepositCrypto,
    WithdrawalCrypto,
    DepositFiat,
    WithdrawalFiat,
    TransferInternal,
    Airdrop,
    StakingReward,
    Expense, // fee-only, manual spend
    GiftIn,
    GiftOut,
    DerivativePnL, // реализованный PnL по перп/фьючерсам
    FundingFee,    // регулярная плата по деривативам (расход)
    Stolen,
    Lost,
    Burn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub asset: AssetId,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub tenant_id: String,
    pub wallet: String, // MEXC || ByBit || OKX || etc
    pub time_utc: chrono::DateTime<chrono::Utc>,

    pub kind: TxKind,
    pub in_money: Option<Money>,
    pub out_money: Option<Money>,
    pub fee_money: Option<Money>,

    pub contract_symbol: Option<String>,         // "BTCUSDT", "ETHUSDT"
    pub derivative_kind: Option<DerivativeKind>, // "perpetual" | "futures"
    pub position_id: Option<String>,             // если даёт биржа

    pub order_id: Option<String>,
    pub tx_hash: Option<String>,
    pub note: Option<String>,

    pub import_id: Uuid,
    pub price_status: PriceStatus,
}
