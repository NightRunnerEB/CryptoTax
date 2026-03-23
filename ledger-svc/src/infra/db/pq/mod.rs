pub mod import_repo;
pub mod tx_repo;
pub mod uow;

#[cfg(test)]
pub(crate) mod test_utils {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        domain::models::{
            import::{Import, ImportStatus},
            transaction::{Asset, Transaction, TxKind},
        },
        infra::db::make_pool,
    };

    pub async fn test_pool() -> PgPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("LEDGER_TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("LEDGER_TEST_DATABASE_URL or DATABASE_URL must be set for integration tests");

        let pool = make_pool(&url, 5, 5).await.expect("connect test database");
        reset_schema(&pool).await;
        pool
    }

    pub async fn reset_schema(pool: &PgPool) {
        // drop old schema objects if they exist
        sqlx::raw_sql(include_str!("../../../../migrations/20251028000300_create_outbox.down.sql"))
            .execute(pool)
            .await
            .expect("drop outbox");
        sqlx::raw_sql(include_str!("../../../../migrations/20251028000200_create_transactions.down.sql"))
            .execute(pool)
            .await
            .expect("drop transactions");
        sqlx::raw_sql(include_str!("../../../../migrations/20251028000100_create_imports.down.sql"))
            .execute(pool)
            .await
            .expect("drop imports");

        // create fresh schema
        sqlx::raw_sql(include_str!("../../../../migrations/20251028000100_create_imports.up.sql"))
            .execute(pool)
            .await
            .expect("create imports");
        sqlx::raw_sql(include_str!("../../../../migrations/20251028000200_create_transactions.up.sql"))
            .execute(pool)
            .await
            .expect("create transactions");
        sqlx::raw_sql(include_str!("../../../../migrations/20251028000300_create_outbox.up.sql"))
            .execute(pool)
            .await
            .expect("create outbox");
    }

    pub fn sample_import(tenant_id: Uuid) -> Import {
        Import {
            id: Uuid::new_v4(),
            tenant_id,
            source: "mexc".to_string(),
            file_name: Some("sample.csv".to_string()),
            status: ImportStatus::Processing,
            total_count: 0,
            error_summary: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    pub fn sample_transaction(tenant_id: Uuid, import_id: Uuid, order_id: &str) -> Transaction {
        use std::str::FromStr;

        use rust_decimal::Decimal;

        Transaction {
            id: Uuid::new_v4(),
            tenant_id,
            import_id,
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
            order_id: Some(order_id.to_string()),
            tx_hash: None,
            note: None,
            time_utc: chrono::Utc::now(),
        }
    }
}
