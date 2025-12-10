CREATE TABLE transaction_tags (
    transaction_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,

    PRIMARY KEY (transaction_id, tag_id),
    
    -- If a tag is deleted, the associated record will also be automatically deleted.
    FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE
);

-- CREATE TABLE IF NOT EXISTS transaction_tags (
--     transaction_id BIGINT NOT NULL,
--     tag_id BIGINT NOT NULL,

--     PRIMARY KEY (transaction_id, tag_id),
    
--     -- If a tag is deleted, the associated record will also be automatically deleted.
--     FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id) ON DELETE CASCADE,
--     FOREIGN KEY (tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE
-- );