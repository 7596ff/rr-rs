use std::fmt::Write;

use anyhow::{anyhow, Result};
use rand::{seq::SliceRandom, thread_rng};
use twilight::model::id::RoleId;

use crate::{
    model::{MessageContext, Response, ResponseReaction},
    reactions,
    table::{Movie, MovieVote},
    util,
};

#[derive(Debug)]
struct Settings {
    movies_role: String,
}

#[derive(Debug)]
struct Nominated {
    nominated: bool,
}

async fn close(context: &MessageContext) -> Result<Response> {
    let movie_votes = sqlx::query_as!(
        MovieVote,
        "DELETE FROM movie_votes WHERE
        (guild_id = $1)
        RETURNING *;",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_all(&context.pool)
    .await?;

    let voted_movies = movie_votes.iter().fold(Vec::new(), |mut acc, m| {
        if !acc.contains(&m.id) {
            acc.push(m.id)
        }
        acc
    });

    for id in voted_movies {
        let votes = movie_votes.iter().filter(|m| m.id == id).count() as i32;
        sqlx::query!(
            "UPDATE movies SET final_votes = $1 WHERE
            (guild_id = $2 AND id = $3);",
            votes,
            context.message.guild_id.unwrap().to_string(),
            id,
        )
        .execute(&context.pool)
        .await?;
    }

    let movies = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE
        (guild_id = $1)
        ORDER BY final_votes DESC;",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_all(&context.pool)
    .await?;

    let highest_vote = movies.iter().fold(0, |mut acc, m| {
        if m.final_votes > acc {
            acc = m.final_votes;
        }

        acc
    });

    let winners: Vec<&Movie> = movies.iter().filter(|m| m.final_votes == highest_vote).collect();

    let mut content = String::new();
    let winner = match winners.len() {
        len if len > 2 => {
            let winner = winners.choose(&mut thread_rng()).unwrap();
            write!(content, "Multiple winners detected. Randomly chose **{}**", winner.title)?;
            *winner
        }
        _ => {
            let winner = winners[0];
            write!(content, "The winner is: **{}**", winner.title)?;
            winner
        }
    };

    if winner.url.is_none() {
        write!(
            content,
            "\n**{}** has no url, please set with `katze movie set-url <url> <title>`",
            winner.title
        )?;
    }

    sqlx::query!("INSERT INTO movie_seq (id) VALUES ($1);", winner.id)
        .execute(&context.pool)
        .await?;

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}

async fn nominate(context: &MessageContext) -> Result<Response> {
    let content = context.args.join(" ");
    let movie = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE
        (guild_id = $1 AND member_id = $2 AND SOUNDEX(title) = SOUNDEX($3))
        LIMIT 1;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        content,
    )
    .fetch_one(&context.pool)
    .await?;

    if movie.title != content && !util::did_you_mean(&context, &movie.title).await? {
        return Err(anyhow!("Movie not found: {}", content));
    }

    sqlx::query!(
        "UPDATE movies SET nominated = FALSE WHERE
        (guild_id = $1 AND member_id = $2 AND title != $3);",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        movie.title,
    )
    .execute(&context.pool)
    .await?;

    let nominated = sqlx::query_as!(
        Nominated,
        "UPDATE movies SET nominated = NOT nominated WHERE
        (guild_id = $1 AND member_id = $2 AND title = $3)
        RETURNING nominated;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        movie.title,
    )
    .fetch_one(&context.pool)
    .await?;

    let response = if nominated.nominated {
        format!("✅ {} is nominated", movie.title)
    } else {
        format!("✅ {} is **no longer** nominated", movie.title)
    };

    let reply = context.reply(response).await?;
    Ok(Response::Message(reply))
}

async fn set_url(context: &mut MessageContext) -> Result<Response> {
    if context.args.is_empty() {
        return Ok(Response::None);
    }

    let url = context.next().ok_or_else(|| anyhow!("Couldn't find movie url"))?;
    let title = context.args.join(" ");

    if title.is_empty() {
        return Err(anyhow!("Couldn't find movie title"));
    }

    sqlx::query!(
        "UPDATE movies SET url = $1 WHERE
        (guild_id = $2 AND title = $3);",
        url,
        context.message.guild_id.unwrap().to_string(),
        title
    )
    .execute(&context.pool)
    .await?;

    context.react(ResponseReaction::Success.value()).await?;
    Ok(Response::Reaction)
}

async fn suggestions_add(context: &MessageContext) -> Result<Response> {
    if context.args.is_empty() {
        return Ok(Response::None);
    }

    sqlx::query!(
        "INSERT INTO movies (guild_id, member_id, title) VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, member_id, title) DO NOTHING;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        context.args.join(" "),
    )
    .execute(&context.pool)
    .await?;

    context.react(ResponseReaction::Success.value()).await?;
    Ok(Response::Reaction)
}

async fn suggestions_list(context: &MessageContext) -> Result<Response> {
    let movies = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE
        (guild_id = $1 AND member_id = $2);",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
    )
    .fetch_all(&context.pool)
    .await?;

    let mut content: String =
        format!("List of suggestions by **{}**\n", context.message.author.name);

    for movie in movies {
        if movie.nominated {
            content.push_str(&format!("- **{}** (nominated)\n", movie.title));
        } else {
            content.push_str(&format!("- {}\n", movie.title));
        }
    }

    content.push_str("Nominate a movie for voting with `katze movie nominate <name>`.");

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}

async fn vote(context: &MessageContext) -> Result<Response> {
    if context.args.is_empty() {
        return reactions::create_menu(&context, "movie_votes").await;
    }

    let content = context.args.join(" ");

    let movie = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE
        (guild_id = $1 AND title = $2 AND nominated);",
        context.message.guild_id.unwrap().to_string(),
        content
    )
    .fetch_one(&context.pool)
    .await?;

    if movie.title != content && !util::did_you_mean(&context, &movie.title).await? {
        return Err(anyhow!("Movie not found: {}", content));
    }

    sqlx::query!(
        "INSERT INTO movie_votes (guild_id, member_id, id) VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, member_id, id) DO
        UPDATE SET id = $3;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        movie.id
    )
    .execute(&context.pool)
    .await?;

    context.react(ResponseReaction::Success.value()).await?;
    Ok(Response::Reaction)
}

pub async fn movie(context: &mut MessageContext) -> Result<Response> {
    let settings = sqlx::query_as!(
        Settings,
        "SELECT movies_role FROM settings WHERE
        (guild_id = $1);",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_one(&context.pool)
    .await?;

    if settings.movies_role.len() > 1 {
        let movies_role = RoleId::from(settings.movies_role.parse::<u64>()?);

        let member = context
            .cache
            .member(context.message.guild_id.unwrap(), context.message.author.id)
            .await?;

        if member.is_some() && !member.unwrap().roles.contains(&movies_role) {
            let reply = context.reply("You do not have the movies role on this server.").await?;
            return Ok(Response::Message(reply));
        }
    }

    match context.next().as_deref() {
        Some("close") => close(context).await,
        Some("nominate") => nominate(context).await,
        Some("set-url") => set_url(context).await,
        Some("suggest") => suggestions_add(context).await,
        Some("suggestions") => match context.next().as_deref() {
            Some("add") => suggestions_add(context).await,
            Some("list") => suggestions_list(context).await,
            _ => suggestions_list(context).await,
        },
        Some("vote") => vote(context).await,
        _ => {
            let reply = context.reply("unknown movie subcommand").await?;
            Ok(Response::Message(reply))
        }
    }
}
