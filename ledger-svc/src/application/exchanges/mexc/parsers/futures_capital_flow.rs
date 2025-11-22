use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use csv_async::StringRecord;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    error::Result,
    models::{
        transaction::{
            AssetId, AssetKind, DerivativeKind, Money, PriceStatus, Transaction, TxKind,
        },
        utils::{HeaderView, ParseContext},
    },
    services::{Parser, ParserFactory},
};

/// Futures > Futures Capital Flow
#[derive(Deserialize, Serialize)]
pub struct FuturesCapitalFlowFactory {
    pub required_headers: Vec<String>,
}

#[typetag::serde]
impl ParserFactory for FuturesCapitalFlowFactory {
    fn id(&self) -> &'static str {
        "mexc.futures.capital_flow"
    }
    fn matches(&self, header: &HeaderView) -> bool {
        header.contains_all(&self.required_headers)
    }
    fn build(&self, header: &HeaderView, ctx: &ParseContext) -> Box<dyn Parser> {
        let mut idx = HashMap::new();
        let mut i;
        for name in &self.required_headers {
            i = header.get(&name).expect("error");
            idx.insert(name.clone(), i);
        }
        Box::new(FuturesCapitalFlowParser {
            idx,
            tenant_id: ctx.tenant_id.clone(),
            import_id: ctx.import_id,
            wallet: ctx.wallet.clone(),
        })
    }
}
pub struct FuturesCapitalFlowParser {
    idx: HashMap<String, usize>,
    tenant_id: String,
    import_id: Uuid,
    wallet: String,
}
impl Parser for FuturesCapitalFlowParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        let get = |name: &str| -> Result<&str> {
            let i = *self.idx.get(name).ok_or_else(|| {
                crate::domain::error::LedgerError::CsvFormat(format!(
                    "missing header index for {}",
                    name
                ))
            })?;
            Ok(row.get(i).unwrap_or("").trim())
        };

        let time_str = get("time")?;
        let pair = get("futures_trading_pair")?;
        let crypto_raw = get("crypto")?;
        let fund_type = get("fund_type")?;
        let flow_type = get("fund_flow_type")?;
        let amount_str = get("amount")?;

        let naive = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").map_err(|e| {
            crate::domain::error::LedgerError::CsvFormat(format!(
                "invalid time '{}': {}",
                time_str, e
            ))
        })?;
        let time_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);

        let amount: Decimal = amount_str.parse().map_err(|e| {
            crate::domain::error::LedgerError::CsvFormat(format!(
                "invalid amount '{}': {}",
                amount_str, e
            ))
        })?;

        let asset_symbol = if crypto_raw.is_empty() {
            "USDT"
        } else {
            crypto_raw
        };
        let usdt = AssetId {
            symbol: asset_symbol.to_string(),
            kind: AssetKind::Crypto,
        };

        let mut kind = TxKind::DerivativePnL;
        let mut in_money: Option<Money> = None;
        let mut out_money: Option<Money> = None;
        let mut fee_money: Option<Money> = None;

        match fund_type {
            // Реализованный PnL по закрытию
            "CLOSE_POSITION" => {
                kind = TxKind::DerivativePnL;
                if amount.is_sign_positive() {
                    in_money = Some(Money {
                        asset: usdt.clone(),
                        amount,
                    });
                } else {
                    out_money = Some(Money {
                        asset: usdt.clone(),
                        amount: -amount,
                    });
                }
            }
            // Комиссии/фандинг как расход
            "FEE" => {
                kind = TxKind::FundingFee;
                let v = if amount.is_sign_negative() {
                    -amount
                } else {
                    amount
                };
                fee_money = Some(Money {
                    asset: usdt.clone(),
                    amount: v,
                });
            }
            // Переводы между кошельками — пропускаем
            "TRANSFER" => return Ok(None),
            _ => {
                // Пропускаем неизвестные типы без ошибки — можно усилить правило позже
                return Ok(None);
            }
        }

        let tx = Transaction {
            id: Uuid::new_v4(),
            tenant_id: self.tenant_id.clone(),
            wallet: self.wallet.clone(),
            time_utc,
            kind,
            in_money,
            out_money,
            fee_money,
            contract_symbol: if pair.is_empty() {
                None
            } else {
                Some(pair.to_string())
            },
            derivative_kind: Some(DerivativeKind::Futures),
            position_id: None,
            order_id: None,
            tx_hash: None,
            note: Some(format!("fund_type={}, flow_type={}", fund_type, flow_type)),
            import_id: self.import_id,
            price_status: PriceStatus::Pending,
        };

        Ok(Some(tx))
    }
    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
