use std::str::FromStr;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::domain::models::transaction::{Asset, DerivativeKind, Transaction, TxKind};

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
    pub in_money: Option<Json<AssetDto>>,
    pub out_money: Option<Json<AssetDto>>,
    pub fee_money: Option<Json<AssetDto>>,
    pub contract_symbol: Option<String>,
    pub derivative_kind: Option<String>,
    pub position_id: Option<String>,
    pub order_id: Option<String>,
    pub tx_hash: Option<String>,
    pub note: Option<String>,
    pub import_id: Uuid,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = String;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: row.id,
            tenant_id: row.tenant_id,
            wallet: row.wallet,
            time_utc: row.time_utc,
            kind: TxKind::from_str(&row.kind)?,
            in_money: row.in_money.map(|Json(a)| Asset::from(a)),
            out_money: row.out_money.map(|Json(a)| Asset::from(a)),
            fee_money: row.fee_money.map(|Json(a)| Asset::from(a)),
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

impl From<&Transaction> for TransactionRow {
    fn from(tx: &Transaction) -> Self {
        TransactionRow {
            id: tx.id,
            tenant_id: tx.tenant_id,
            wallet: tx.wallet.clone(),
            time_utc: tx.time_utc,
            kind: tx.kind.to_string(),
            in_money: tx.in_money.clone().map(|a| Json(AssetDto::from(a))),
            out_money: tx.out_money.clone().map(|a| Json(AssetDto::from(a))),
            fee_money: tx.fee_money.clone().map(|a| Json(AssetDto::from(a))),
            contract_symbol: tx.contract_symbol.clone(),
            derivative_kind: tx.derivative_kind.map(|d| d.to_string()),
            position_id: tx.position_id.clone(),
            order_id: tx.order_id.clone(),
            tx_hash: tx.tx_hash.clone(),
            note: tx.note.clone(),
            import_id: tx.import_id,
        }
    }
}
