CREATE TABLE IF NOT EXISTS currency_rates (
    currency TEXT PRIMARY KEY,
    rate_to_base TEXT NOT NULL  -- example: CAD=1.0, USD=0.72
);