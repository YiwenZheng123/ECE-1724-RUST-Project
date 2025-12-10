CREATE TABLE IF NOT EXISTS savings_goals (
    goal_id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    goal_name TEXT NOT NULL,
    target_amount TEXT NOT NULL,
    current_amount TEXT NOT NULL DEFAULT 0,
    deadline TEXT NOT NULL,

    FOREIGN KEY (account_id) REFERENCES accounts(account_id) ON DELETE CASCADE
);

-- CREATE TABLE IF NOT EXISTS savings_goals (
--     goal_id BIGSERIAL PRIMARY KEY,
--     account_id BIGINT NOT NULL,
--     goal_name TEXT NOT NULL,
--     target_amount NUMERIC(12, 2) NOT NULL,
--     current_amount NUMERIC(12, 2) NOT NULL DEFAULT 0,
--     deadline TIMESTAMP NOT NULL,

--     FOREIGN KEY (account_id) REFERENCES accounts(account_id) ON DELETE CASCADE
-- );
