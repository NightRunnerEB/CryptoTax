use std::{fmt, str::FromStr};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub symbol: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub import_id: Uuid,
    pub wallet: String, // MEXC || ByBit || OKX || etc

    pub kind: TxKind,
    pub in_money: Option<Asset>,
    pub out_money: Option<Asset>,
    pub fee_money: Option<Asset>,

    pub contract_symbol: Option<String>, // "BTCUSDT", "ETHUSDT"
    pub derivative_kind: Option<DerivativeKind>, // "perpetual" | "futures"
    pub position_id: Option<String>,     // if exchanges provide

    pub order_id: Option<String>,
    pub tx_hash: Option<String>,
    pub note: Option<String>,

    pub time_utc: chrono::DateTime<chrono::Utc>,
}

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

impl fmt::Display for TxKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TxKind::Spot => "Spot",
            TxKind::Swap => "Swap",
            TxKind::DepositCrypto => "DepositCrypto",
            TxKind::WithdrawalCrypto => "WithdrawalCrypto",
            TxKind::DepositFiat => "DepositFiat",
            TxKind::WithdrawalFiat => "WithdrawalFiat",
            TxKind::TransferInternal => "TransferInternal",
            TxKind::Airdrop => "Airdrop",
            TxKind::StakingReward => "StakingReward",
            TxKind::Expense => "Expense",
            TxKind::GiftIn => "GiftIn",
            TxKind::GiftOut => "GiftOut",
            TxKind::DerivativePnL => "DerivativePnL",
            TxKind::FundingFee => "FundingFee",
            TxKind::Stolen => "Stolen",
            TxKind::Lost => "Lost",
            TxKind::Burn => "Burn",
        };
        f.write_str(s)
    }
}

impl FromStr for TxKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TxKind::*;
        match s {
            "Spot" => Ok(Spot),
            "Swap" => Ok(Swap),
            "DepositCrypto" => Ok(DepositCrypto),
            "WithdrawalCrypto" => Ok(WithdrawalCrypto),
            "DepositFiat" => Ok(DepositFiat),
            "WithdrawalFiat" => Ok(WithdrawalFiat),
            "TransferInternal" => Ok(TransferInternal),
            "Airdrop" => Ok(Airdrop),
            "StakingReward" => Ok(StakingReward),
            "Expense" => Ok(Expense),
            "GiftIn" => Ok(GiftIn),
            "GiftOut" => Ok(GiftOut),
            "DerivativePnL" => Ok(DerivativePnL),
            "FundingFee" => Ok(FundingFee),
            "Stolen" => Ok(Stolen),
            "Lost" => Ok(Lost),
            "Burn" => Ok(Burn),
            other => Err(format!("unknown TxKind: {other}")),
        }
    }
}

impl fmt::Display for DerivativeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DerivativeKind::Perpetual => "Perpetual",
            DerivativeKind::Futures => "Futures",
            DerivativeKind::Option => "Option",
            DerivativeKind::Leveraged => "Leveraged",
        };
        f.write_str(s)
    }
}

impl FromStr for DerivativeKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DerivativeKind::*;
        match s {
            "Perpetual" => Ok(Perpetual),
            "Futures" => Ok(Futures),
            "Option" => Ok(Option),
            "Leveraged" => Ok(Leveraged),
            other => Err(format!("unknown DerivativeKind: {other}")),
        }
    }
}
