PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;

-- 账户（先清空）
DELETE FROM accounts;
DELETE FROM sqlite_sequence WHERE name='accounts';

INSERT INTO accounts (account_name, account_type, balance, currency, account_created_at) VALUES
  ('Checking RBC', 'checking', '0.00', 'CAD', datetime('now')),
  ('Cash Wallet',  'cash',     '0.00', 'CAD', datetime('now'));

-- 类目（保幂等，避免重复）
CREATE UNIQUE INDEX IF NOT EXISTS ux_categories_name_type
ON categories(LOWER(category_name), category_type);

INSERT OR IGNORE INTO categories (category_name, category_type, icon) VALUES
  ('Salary',  'Income',  ''),
  ('Rent',    'Expense', ''),
  ('Transport','Expense','');

-- 交易（全部正额；is_expense=1 表示支出，=0 表示收入）
DELETE FROM transactions;
DELETE FROM sqlite_sequence WHERE name='transactions';

-- 1) 工资入账（Checking，收入）
INSERT INTO transactions (account_id, category_id, amount, is_expense, description, currency, transacted_at)
VALUES (
  (SELECT account_id FROM accounts WHERE account_name='Checking RBC'),
  (SELECT category_id FROM categories WHERE category_name='Salary' AND category_type='Income'),
  '3800.00', 0, 'Monthly salary', 'CAD', '2025-12-01 09:00:00'
);

-- 2) 房租（Checking，支出）
INSERT INTO transactions (account_id, category_id, amount, is_expense, description, currency, transacted_at)
VALUES (
  (SELECT account_id FROM accounts WHERE account_name='Checking RBC'),
  (SELECT category_id FROM categories WHERE category_name='Rent' AND category_type='Expense'),
  '1800.00', 1, 'December rent', 'CAD', '2025-12-02 08:30:00'
);

-- 3) 地铁一次（Cash，支出）
INSERT INTO transactions (account_id, category_id, amount, is_expense, description, currency, transacted_at)
VALUES (
  (SELECT account_id FROM accounts WHERE account_name='Cash Wallet'),
  (SELECT category_id FROM categories WHERE category_name='Transport' AND category_type='Expense'),
  '3.35', 1, 'TTC single fare', 'CAD', '2025-12-02 08:15:00'
);

COMMIT;

-- 基于交易回算各账户余额（支出为负，收入为正）
UPDATE accounts
SET balance = IFNULL((
  SELECT ROUND(SUM(
    CASE WHEN t.is_expense = 1
         THEN -CAST(t.amount AS NUMERIC)
         ELSE  CAST(t.amount AS NUMERIC)
    END
  ), 2)
  FROM transactions t
  WHERE t.account_id = accounts.account_id
), 0.00);
