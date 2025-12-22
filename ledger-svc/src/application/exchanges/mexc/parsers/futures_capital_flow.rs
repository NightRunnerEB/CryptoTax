use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use csv_async::StringRecord;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    error::{LedgerError, Result},
    models::{
        transaction::{Asset, DerivativeKind, Transaction, TxKind},
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

        println!("{:#?}", idx);

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
    tenant_id: Uuid,
    import_id: Uuid,
    wallet: String,
}

impl Parser for FuturesCapitalFlowParser {
    fn push(&mut self, row: &StringRecord) -> Result<Option<Transaction>> {
        let get = |name: &str| -> Result<&str> {
            let i = *self.idx.get(name).ok_or_else(|| LedgerError::CsvFormat(format!("missing header index for {}", name)))?;
            Ok(row.get(i).unwrap_or("").trim())
        };

        let time_str = get("time")?;
        let pair = get("futures_trading_pair")?;
        let crypto_raw = get("crypto")?;
        let fund_type = get("fund_type")?; // CLOSE_POSITION | FEE | TRANSFER | ...
        let flow_type = get("fund_flow_type")?; // инфо для заметки
        let amount_str = get("amount")?;

        let naive = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| LedgerError::CsvFormat(format!("invalid time '{}': {}", time_str, e)))?;
        let time_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);

        let amount: Decimal =
            amount_str.parse().map_err(|e| LedgerError::CsvFormat(format!("invalid amount '{}': {}", amount_str, e)))?;

        // Пустое поле crypto = расчёты в USDT
        let asset_symbol = if crypto_raw.is_empty() {
            "USDT"
        } else {
            crypto_raw
        };

        let mut kind = TxKind::DerivativePnL;
        let mut in_money: Option<Asset> = None;
        let mut out_money: Option<Asset> = None;
        let mut fee_money: Option<Asset> = None;

        match fund_type {
            // Реализованный PnL по закрытию позиций:
            // >0 — прибыль (in), <0 — убыток (out)
            "CLOSE_POSITION" => {
                kind = TxKind::DerivativePnL;
                if amount.is_sign_positive() {
                    in_money = Some(Asset {
                        symbol: asset_symbol.to_string(),
                        amount,
                    });
                } else if amount.is_zero() {
                    // нулевая строка — пропускаем
                    return Ok(None);
                } else {
                    out_money = Some(Asset {
                        symbol: asset_symbol.to_string(),
                        amount: -amount,
                    });
                }
            }

            // Фандинг/комиссии — это расход: кладём в fee_money (а не out_money),
            // чтобы отчёты/агрегации по комиссиям читались корректно.
            "FEE" => {
                kind = TxKind::FundingFee;
                if amount.is_zero() {
                    return Ok(None);
                }
                fee_money = Some(Asset {
                    symbol: asset_symbol.to_string(),
                    amount: amount.abs(),
                });
            }

            // Внутренние переводы биржи — игнорируем как не влияющие на налоговую базу
            "TRANSFER" => return Ok(None),

            // Прочее — мягко пропускаем (можно ужесточить до ошибки позже)
            _ => return Ok(None),
        }

        let tx = Transaction {
            id: Uuid::new_v4(),
            tenant_id: self.tenant_id,
            import_id: self.import_id,
            source: self.wallet.clone(),

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

            time_utc,
        };

        Ok(Some(tx))
    }

    fn finish(self: Box<Self>) -> Result<Vec<Transaction>> {
        Ok(vec![])
    }
}
