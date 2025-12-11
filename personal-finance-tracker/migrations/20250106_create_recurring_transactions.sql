CREATE TABLE recurring_transactions (
    recurring_id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    amount TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'CAD',
    category_id INTEGER,
    description TEXT,
    recurrence_rule TEXT NOT NULL, -- e.g., 'daily', 'weekly', 'monthly'
    next_run_date TEXT NOT NULL,

    FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES categories(category_id) ON DELETE CASCADE
);

-- CREATE TABLE recurring_transactions (
--     recurring_id BIGSERIAL PRIMARY KEY,
--     account_id BIGINT NOT NULL,
--     amount NUMERIC(12, 2) NOT NULL,
--     currency TEXT NOT NULL DEFAULT 'CAD',
--     category_id BIGINT,
--     description TEXT,
--     recurrence_rule TEXT NOT NULL, -- e.g., 'daily', 'weekly', 'monthly'
--     next_run_date TIMESTAMP NOT NULL,

--     FOREIGN KEY(account_id) REFERENCES accounts(account_id) ON DELETE CASCADE,
--     FOREIGN KEY(category_id) REFERENCES categories(category_id) ON DELETE CASCADE
-- );
