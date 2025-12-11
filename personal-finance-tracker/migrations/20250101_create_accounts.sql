CREATE TABLE IF NOT EXISTS accounts (
    account_id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    balance TEXT NOT NULL,
    currency TEXT NOT NULL,
    account_created_at TEXT NOT NULL
);

-- CREATE TABLE IF NOT EXISTS accounts (
--     account_id BIGSERIAL PRIMARY KEY,
--     account_name TEXT NOT NULL,
--     account_type TEXT NOT NULL,
--     balance NUMERIC(12, 2) NOT NULL,
--     currency TEXT NOT NULL,
--     account_created_at TIMESTAMP NOT NULL DEFAULT NOW()
-- );
