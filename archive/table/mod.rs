#[derive(Debug)]
pub struct Movie {
    pub guild_id: SqlxGuildId,
    pub member_id: SqlxUserId,
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub watch_date: Option<NaiveDateTime>,
    pub nominated: bool,
    pub final_votes: i32,
}

#[derive(Debug)]
pub struct MovieVote {
    pub guild_id: SqlxGuildId,
    pub member_id: SqlxUserId,
    pub id: i32,
}

#[derive(Debug)]
pub struct MovieSeq {
    pub id: i32,
    pub seq: i32,
}
