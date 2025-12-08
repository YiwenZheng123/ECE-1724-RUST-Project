use chrono::NaiveDate;
use rust_decimal::Decimal;

pub fn fmt_money(d: &Decimal) -> String {
    d.round_dp(2).to_string()
}

pub fn parse_money(s: &str) -> Option<Decimal> {
    Decimal::from_str_exact(s.trim()).ok()
}

pub fn parse_date_any(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
        .unwrap_or_else(|_| chrono::Utc::now().date_naive())
}

pub fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

pub fn iso(d: &NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}
