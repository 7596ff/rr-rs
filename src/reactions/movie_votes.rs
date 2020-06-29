use std::str;

use anyhow::{anyhow, Result};
use sqlx::PgPool;
use tokio::stream::StreamExt;
use twilight::{
    builders::embed::EmbedBuilder,
    model::channel::{embed::Embed, ReactionType},
};

use crate::model::{MessageContext, ReactionContext, Response};

#[derive(Clone, Debug)]
pub struct MovieVotes {
    pub id: i32,
    pub title: String,
    pub member_id: String,
    pub count: i64,
}

async fn query(pool: &PgPool, guild_id: String) -> Result<Vec<MovieVotes>> {
    let movies = sqlx::query_as!(
        MovieVotes,
        "SELECT m.id, m.title, m.member_id, COUNT(v.id)
        FROM movies m
        LEFT JOIN movie_votes v ON m.id = v.id
        WHERE (m.guild_id = $1 AND m.nominated)
        GROUP BY m.id, m.title
        ORDER BY m.id;",
        guild_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(movies)
}

pub fn format_menu(data: &[(String, &MovieVotes)]) -> Result<Embed> {
    let mut description: String =
        "Vote for a movie by reacting with its associated number:\n\n".into();

    for (emoji, movie) in data {
        description.push_str(&format!(
            "{} **{}** (<@{}>, votes: {})\n",
            emoji,
            movie.title.clone(),
            movie.member_id.clone(),
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
    let movies = query(&context.pool, context.message.guild_id.unwrap().to_string()).await?;

    let data = movies.iter().enumerate().fold(Vec::new(), |mut acc, (counter, movie)| {
        // one-indexed for 1..9 emojis
        if counter + 1 < 10 {
            let emoji = format!("{}⃣", counter + 1);
            acc.push((emoji, movie));
        }

        acc
    });

    // create the embed
    let embed = format_menu(&data)?;

    // send the message, and react to it
    let sent = context.http.create_message(context.message.channel_id).embed(embed)?.await?;

    for (emoji, _) in &data {
        let emoji = ReactionType::Unicode { name: emoji.to_string() };
        context.http.create_reaction(context.message.channel_id, sent.id, emoji).await?;
    }

    // create emoji mapping for storage in redis
    let mapping: Vec<(String, i32)> = data.iter().map(|(e, m)| (e.clone(), m.id)).collect();

    let key = format!("reaction_menu:{}:{}:movie_votes", sent.channel_id, sent.id);
    let mapping = serde_json::to_string(&mapping)?;
    redis.set(key, mapping).await?;

    Ok(Response::Message(sent))
}

pub async fn handle_event(context: &ReactionContext) -> Result<()> {
    let key = format!(
        "reaction_menu:{}:{}:movie_votes",
        context.reaction.channel_id, context.reaction.message_id
    );

    let mut redis = context.redis.get().await;
    let mapping = redis.get(&key).await?.ok_or_else(|| anyhow!("redis: key {} not found", &key))?;

    let mapping: Vec<(String, i32)> = serde_json::from_str(str::from_utf8(&mapping)?).unwrap();

    let reaction = mapping.iter().find(|&(emoji, _)| match &context.reaction.emoji {
        ReactionType::Unicode { name } => emoji == name,
        _ => false,
    });

    if let Some(reaction) = reaction {
        sqlx::query!(
            "DELETE FROM movie_votes WHERE
            (guild_id = $1 AND member_id = $2);",
            context.reaction.guild_id.unwrap().to_string(),
            context.reaction.user_id.to_string(),
        )
        .execute(&context.pool)
        .await?;

        sqlx::query!(
            "INSERT INTO movie_votes (guild_id, member_id, id) VALUES ($1, $2, $3)
            ON CONFLICT (guild_id, member_id, id) DO NOTHING;",
            context.reaction.guild_id.unwrap().to_string(),
            context.reaction.user_id.to_string(),
            reaction.1
        )
        .execute(&context.pool)
        .await?;
    }

    let movies = query(&context.pool, context.reaction.guild_id.unwrap().to_string()).await?;

    let data: Vec<(String, &MovieVotes)> =
        mapping.iter().map(|tuple| tuple.0.clone()).zip(movies.iter()).collect();

    let embed = format_menu(&data)?;

    context
        .http
        .update_message(context.reaction.channel_id, context.reaction.message_id)
        .embed(embed)?
        .await?;

    Ok(())
}
