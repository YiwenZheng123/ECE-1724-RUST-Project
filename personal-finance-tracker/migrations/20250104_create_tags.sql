CREATE TABLE IF NOT EXISTS tags (
    tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag TEXT NOT NULL,
    FOREIGN KEY(tag_id) REFERENCES transactions(transaction_id)
);
