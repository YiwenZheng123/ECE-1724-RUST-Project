CREATE TABLE IF NOT EXISTS categories (
    category_id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_name TEXT NOT NULL,
    category_type TEXT NOT NULL,
    icon TEXT NOT NULL
);

-- CREATE TABLE IF NOT EXISTS categories (
--     category_id BIGSERIAL PRIMARY KEY,
--     category_name TEXT NOT NULL,
--     category_type TEXT NOT NULL,
--     icon TEXT NOT NULL
-- );
