CREATE TABLE IF NOT EXISTS budgets (
    budget_id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    category_id INTEGER,
    period TEXT NOT NULL,        -- 'weekly' / 'monthly'
    amount TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'CAD',
    start_date TEXT NOT NULL,    -- first day of cycle

    FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE
);

-- CREATE TABLE IF NOT EXISTS budgets (
--     budget_id BIGSERIAL PRIMARY KEY,
--     account_id BIGINT NOT NULL,
--     category_id BIGINT NOT NULL,
--     period TEXT NOT NULL,        -- 'weekly' / 'monthly'
--     amount NUMERIC(12, 2) NOT NULL,
--     currency TEXT NOT NULL DEFAULT 'CAD',
--     start_date TIMESTAMP NOT NULL,    -- first day of cycle

--     FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE
-- );
