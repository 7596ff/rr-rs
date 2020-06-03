use anyhow::Result;
use tokio::stream::StreamExt;
use twilight::{
    builders::embed::EmbedBuilder,
    model::channel::{embed::Embed, ReactionType},
};

use crate::model::{MessageContext, Response};

#[derive(Clone, Debug)]
pub struct MovieVotes {
    pub id: i32,
    pub title: String,
    pub count: i64,
}

async fn query(context: &MessageContext) -> Result<Vec<MovieVotes>> {
    let movies = sqlx::query_as!(
        MovieVotes,
        "SELECT m.id, m.title, COUNT(v.id)
        FROM movies m
        LEFT JOIN movie_votes v ON m.id = v.id
        WHERE (m.guild_id = $1 AND m.nominated)
        GROUP BY m.id, m.title
        ORDER BY m.id;",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_all(&context.pool)
    .await?;

    Ok(movies)
}

pub fn format_menu(data: &Vec<(String, &MovieVotes)>) -> Result<Embed> {
    let mut description: String =
        "Vote for a movie by reacting with its associated number:\n\n".into();

    for (emoji, movie) in data {
        description.push_str(&format!(
            "{} **{}** (votes: {})\n",
            emoji,
            movie.title.clone(),
            movie.count.clone()
        ));
    }

    Ok(EmbedBuilder::new().description(description).build())
}

pub async fn create_menu(context: &MessageContext) -> Result<Response> {
    // remove all other reaction menus from the channel
    let delete_pattern = format!("reaction_menu:{}*", context.message.channel_id);

    let mut redis = context.redis.get().await;
    let to_delete = redis.scan().pattern(&delete_pattern).run();
    let to_delete = to_delete.collect::<Vec<Vec<u8>>>().await;

    for key in to_delete {
        redis.del(key).await?;
    }

    // collect the data required to create the reaction menu
    let movies = query(context).await?;
    let data = movies
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut acc, (counter, movie)| {
            // one-indexed for 1..9 emojis
            if counter + 1 < 10 {
                let emoji = format!("{}⃣", counter + 1);
                acc.push((emoji.to_string(), movie));
            }

            acc
        });

    // create the embed
    let embed = format_menu(&data)?;

    // send the message, and react to it
    let sent = context
        .http
        .create_message(context.message.channel_id)
        .embed(embed)?
        .await?;

    for (emoji, _) in &data {
        let emoji = ReactionType::Unicode {
            name: emoji.to_string(),
        };
        context
            .http
            .create_reaction(context.message.channel_id, sent.id, emoji)
            .await?;
    }

    // create emoji mapping for storage in redis
    let mapping: Vec<(String, i32)> = data
        .iter()
        .map(|(e, m)| (e.clone(), m.id.clone()))
        .collect();

    let key = format!("reaction_menu:{}:{}:movie_votes", sent.channel_id, sent.id);
    let mapping = serde_json::to_string(&mapping)?;
    redis.set(key, mapping).await?;

    Ok(Response::Some(sent))
}

// let mapping: Vec<(String, i32)> = serde_json::from_str(redis_result).unwrap();
pub async fn handle_event(context: &MessageContext) {}
