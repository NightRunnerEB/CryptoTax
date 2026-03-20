use std::{fmt, str::FromStr};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::error;
use uuid::Uuid;

use crate::domain::error::LedgerError;

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
    pub source: String, // MEXC || ByBit || OKX || etc

    pub kind: TxKind,
    pub in_money: Option<Asset>,
    pub out_money: Option<Asset>,
    pub fee_money: Option<Asset>,

    pub contract_symbol: Option<String>,         // "BTCUSDT", "ETHUSDT"
    pub derivative_kind: Option<DerivativeKind>, // "perpetual" | "futures"
    pub position_id: Option<String>,             // if exchanges provide

    pub order_id: Option<String>,
    pub tx_hash: Option<String>,
    pub note: Option<String>,

    pub time_utc: chrono::DateTime<chrono::Utc>,
}

impl Transaction {
    pub fn compute_fingerprint(&self) -> String {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.tenant_id,
            self.source,
            self.time_utc.to_rfc3339(),
            self.kind,
            self.in_money.as_ref().map_or("".to_string(), |a| format!("{}:{}", a.symbol, a.amount)),
            self.out_money.as_ref().map_or("".to_string(), |a| format!("{}:{}", a.symbol, a.amount)),
            self.fee_money.as_ref().map_or("".to_string(), |a| format!("{}:{}", a.symbol, a.amount)),
            self.contract_symbol.as_deref().unwrap_or(""),
            self.order_id.as_deref().unwrap_or(""),
            self.position_id.as_deref().unwrap_or(""),
            self.tx_hash.as_deref().unwrap_or(""),
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let digest = hasher.finalize();
        hex::encode(digest)
    }
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
    type Err = LedgerError;

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
            other => {
                error!("unknown TxKind: {other}");
                Err(LedgerError::Internal)
            }
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
    type Err = LedgerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DerivativeKind::*;
        match s {
            "Perpetual" => Ok(Perpetual),
            "Futures" => Ok(Futures),
            "Option" => Ok(Option),
            "Leveraged" => Ok(Leveraged),
            other => {
                error!("unknown DerivativeKind: {other}");
                Err(LedgerError::Internal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use super::{Asset, Transaction, TxKind};
    use crate::domain::error::LedgerError;

    fn make_tx() -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            import_id: Uuid::new_v4(),
            source: "MEXC".to_string(),
            kind: TxKind::Spot,
            in_money: Some(Asset {
                symbol: "BTC".to_string(),
                amount: Decimal::from_str("0.1").expect("decimal"),
            }),
            out_money: Some(Asset {
                symbol: "USDT".to_string(),
                amount: Decimal::from_str("3000").expect("decimal"),
            }),
            fee_money: None,
            contract_symbol: None,
            derivative_kind: None,
            position_id: None,
            order_id: Some("order-1".to_string()),
            tx_hash: None,
            note: None,
            time_utc: Utc::now(),
        }
    }

    #[test]
    fn fingerprint_is_stable_for_same_transaction_data() {
        let tx = make_tx();
        let fp1 = tx.compute_fingerprint();
        let fp2 = tx.compute_fingerprint();
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn fingerprint_changes_when_business_fields_change() {
        let mut tx1 = make_tx();
        let fp1 = tx1.compute_fingerprint();

        tx1.order_id = Some("order-2".to_string());
        let fp2 = tx1.compute_fingerprint();

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn tx_kind_rejects_unknown_value() {
        let err = TxKind::from_str("UnknownKind").expect_err("unknown kind should fail");
        assert!(matches!(err, LedgerError::Internal));
    }
}
