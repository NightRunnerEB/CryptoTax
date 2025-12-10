CREATE TABLE
    transactions (
        id uuid PRIMARY KEY,
        tenant_id uuid NOT NULL,
        wallet text NOT NULL,
        time_utc timestamptz NOT NULL,
        kind text NOT NULL,
        in_money jsonb,
        out_money jsonb,
        fee_money jsonb,
        contract_symbol text,
        derivative_kind text,
        position_id text,
        order_id text,
        tx_hash text,
        note text,
        import_id uuid NOT NULL,
        tx_fingerprint text NOT NULL,
        created_at timestamptz NOT NULL DEFAULT now ()
    );

ALTER TABLE transactions ADD CONSTRAINT fk_transactions_import FOREIGN KEY (import_id) REFERENCES imports (id) ON DELETE RESTRICT; 
-- В БУДУЩЕМ НУЖНО ЗАМЕНИТЬ RESTRICT НА CASCADE, КОГДА БУДЕТ РЕАЛИЗОВАНА ЛОГИКА УДАЛЕНИЯ ИМПОРТОВ В СЕРВИСЕ.

ALTER TABLE transactions ADD CONSTRAINT chk_transactions_kind CHECK (
    kind IN (
        'Spot',
        'Swap',
        'DepositCrypto',
        'WithdrawalCrypto',
        'DepositFiat',
        'WithdrawalFiat',
        'TransferInternal',
        'Airdrop',
        'StakingReward',
        'Expense',
        'GiftIn',
        'GiftOut',
        'DerivativePnL',
        'FundingFee',
        'Stolen',
        'Lost',
        'Burn'
    )
);

CREATE UNIQUE INDEX ux_transactions_fingerprint ON transactions (tx_fingerprint);

CREATE INDEX idx_transactions_import_id ON transactions (import_id);

CREATE INDEX idx_transactions_tenant_time ON transactions (tenant_id, time_utc DESC);