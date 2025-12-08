// each transcation can have multiple tags

use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct Tag {
    pub tag_id: i64,                //key
    pub tag: String,
}