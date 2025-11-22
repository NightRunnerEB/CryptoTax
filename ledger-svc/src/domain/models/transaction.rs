use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivativeKind {
    Perpetual,
    Futures,
    Option,
    Leveraged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct Asset {
    pub symbol: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub wallet: String, // MEXC || ByBit || OKX || etc
    pub time_utc: chrono::DateTime<chrono::Utc>,

    pub kind: TxKind,
    pub in_money: Option<Asset>,
    pub out_money: Option<Asset>,
    pub fee_money: Option<Asset>,

    pub contract_symbol: Option<String>,         // "BTCUSDT", "ETHUSDT"
    pub derivative_kind: Option<DerivativeKind>, // "perpetual" | "futures"
    pub position_id: Option<String>,             // если даёт биржа

    pub order_id: Option<String>,
    pub tx_hash: Option<String>,
    pub note: Option<String>,

    pub import_id: Uuid,
}
