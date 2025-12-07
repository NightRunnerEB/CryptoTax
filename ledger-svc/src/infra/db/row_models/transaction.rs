use std::str::FromStr;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    error::{LedgerError, Result},
    models::transaction::{Asset, DerivativeKind, Transaction, TxKind},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDto {
    pub symbol: String,
    pub amount: Decimal,
}

impl From<Asset> for AssetDto {
    fn from(a: Asset) -> Self {
        Self {
            symbol: a.symbol,
            amount: a.amount,
        }
    }
}
impl From<AssetDto> for Asset {
    fn from(a: AssetDto) -> Self {
        Self {
            symbol: a.symbol,
            amount: a.amount,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TransactionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub wallet: String,
    pub time_utc: DateTime<Utc>,
    pub kind: String,
    pub in_money: Option<serde_json::Value>,
    pub out_money: Option<serde_json::Value>,
    pub fee_money: Option<serde_json::Value>,
    pub contract_symbol: Option<String>,
    pub derivative_kind: Option<String>,
    pub position_id: Option<String>,
    pub order_id: Option<String>,
    pub tx_hash: Option<String>,
    pub note: Option<String>,
    pub import_id: Uuid,
    pub tx_fingerprint: String,
}

fn decode_asset(opt: Option<serde_json::Value>) -> Result<Option<Asset>> {
    match opt {
        None => Ok(None),
        Some(v) => {
            let asset: Asset = serde_json::from_value(v).map_err(|e| {
                let msg = format!("failed to deserialize Asset from JSON: {e}");
                error!("{}", msg);
                LedgerError::Internal
            })?;
            Ok(Some(asset))
        }
    }
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = LedgerError;

    fn try_from(row: TransactionRow) -> std::result::Result<Self, Self::Error> {
        Ok(Transaction {
            id: row.id,
            tenant_id: row.tenant_id,
            wallet: row.wallet,
            time_utc: row.time_utc,
            kind: TxKind::from_str(&row.kind)?,
            in_money: decode_asset(row.in_money)?,
            out_money: decode_asset(row.out_money)?,
            fee_money: decode_asset(row.fee_money)?,
            contract_symbol: row.contract_symbol,
            derivative_kind: match row.derivative_kind {
                Some(s) => Some(DerivativeKind::from_str(&s)?),
                None => None,
            },
            position_id: row.position_id,
            order_id: row.order_id,
            tx_hash: row.tx_hash,
            note: row.note,
            import_id: row.import_id,
        })
    }
}

fn encode_asset(opt: &Option<Asset>) -> Option<serde_json::Value> {
    opt.as_ref().map(|a| serde_json::to_value(a).expect("failed to serialize Asset to JSON"))
}

impl From<&Transaction> for TransactionRow {
    fn from(tx: &Transaction) -> Self {
        TransactionRow {
            id: tx.id,
            tenant_id: tx.tenant_id,
            wallet: tx.wallet.clone(),
            time_utc: tx.time_utc,
            kind: tx.kind.to_string(),
            in_money: encode_asset(&tx.in_money),
            out_money: encode_asset(&tx.out_money),
            fee_money: encode_asset(&tx.fee_money),
            contract_symbol: tx.contract_symbol.clone(),
            derivative_kind: tx.derivative_kind.map(|d| d.to_string()),
            position_id: tx.position_id.clone(),
            order_id: tx.order_id.clone(),
            tx_hash: tx.tx_hash.clone(),
            note: tx.note.clone(),
            import_id: tx.import_id,
            tx_fingerprint: tx.compute_fingerprint(),
        }
    }
}
