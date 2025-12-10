CREATE TABLE IF NOT EXISTS transactions (
    transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    amount TEXT NOT NULL,
    base_amount DOUBLE PRECISION NOT NULL,
    is_expense INTEGER NOT NULL,
    description TEXT,
    currency TEXT,
    transacted_at TEXT NOT NULL,
    trans_create_at TEXT,
    FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES categories(category_id) ON DELETE CASCADE
);

-- CREATE TABLE IF NOT EXISTS transactions (
--     transaction_id BIGSERIAL PRIMARY KEY,
--     account_id BIGINT NOT NULL,
--     category_id BIGINT NOT NULL,
--     amount NUMERIC(12, 2) NOT NULL,
--     base_amount DOUBLE PRECISION NOT NULL,
--     is_expense BOOLEAN NOT NULL,
--     description TEXT,
--     currency TEXT,
--     transacted_at TIMESTAMP NOT NULL,
--     trans_create_at TIMESTAMP DEFAULT NOW(),
--     FOREIGN KEY (account_id) REFERENCES accounts(account_id) ON DELETE CASCADE,
--     FOREIGN KEY (category_id) REFERENCES categories(category_id) ON DELETE CASCADE
-- );
