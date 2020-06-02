use anyhow::{anyhow, Result};

use crate::{
    model::{MessageContext, Response},
    reactions,
    table::Movie,
    util,
};

#[derive(Debug)]
struct Nominated {
    nominated: bool,
}

async fn nominate(context: &MessageContext, content: String) -> Result<Response> {
    let movies = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE
        (guild_id = $1 AND member_id = $2);",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
    )
    .fetch_all(&context.pool)
    .await?;

    let movie_titles: Vec<String> = movies.iter().map(|m| m.title.clone()).collect::<Vec<_>>();
    if !movie_titles.contains(&content) {
        return Err(anyhow!("Couldn't find movie: {}", content));
    }

    let nominated = sqlx::query_as!(
        Nominated,
        "UPDATE movies SET nominated = NOT nominated
        WHERE (guild_id = $1 AND member_id = $2 AND title = $3)
        RETURNING nominated;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        content
    )
    .fetch_one(&context.pool)
    .await?;

    let response = match nominated.nominated {
        true => format!("✅ {} is nominated", content),
        false => format!("✅ {} is **no longer** nominated", content),
    };

    Ok(util::construct_response(
        context
            .http
            .create_message(context.message.channel_id)
            .content(response)
            .await,
    ))
}

async fn set_url(context: &MessageContext, content: String) -> Result<Response> {
    if content.len() < 1 {
        return Ok(Response::None);
    }

    let mut content = content.split(" ");
    let url = content.next().ok_or(anyhow!("Couldn't find movie url"))?;
    let title = content.collect::<Vec<_>>().join(" ");

    if title.len() < 1 {
        return Err(anyhow!("Couldn't find movie title"));
    }

    sqlx::query!(
        "UPDATE movies SET url = $1
        WHERE (guild_id = $2 AND title = $3);",
        url,
        context.message.guild_id.unwrap().to_string(),
        title
    )
    .execute(&context.pool)
    .await?;

    Ok(Response::Reaction("✅".into()))
}

async fn suggestions_add(context: &MessageContext, content: String) -> Result<Response> {
    if content.len() < 1 {
        return Ok(Response::None);
    }

    sqlx::query!(
        "INSERT INTO movies (guild_id, member_id, title) VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, member_id, title) DO NOTHING;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        content,
    )
    .execute(&context.pool)
    .await?;

    Ok(Response::Reaction("✅".into()))
}

async fn suggestions_list(context: &MessageContext) -> Result<Response> {
    let movies = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE guild_id = $1;",
        context.message.guild_id.unwrap().to_string()
    )
    .fetch_all(&context.pool)
    .await?;

    let mut content: String = format!(
        "List of suggestions by **{}**\n",
        context.message.author.name
    )
    .into();

    for movie in movies {
        content.push_str(&format!("- {}\n", movie.title));
    }

    content.push_str("Nominate a movie for voting with `katze movie nominate <name>`.");

    Ok(util::construct_response(
        context
            .http
            .create_message(context.message.channel_id)
            .content(content)
            .await,
    ))
}

async fn vote(context: &MessageContext, content: String) -> Result<Response> {
    if content.len() < 1 {
        return reactions::create_menu(&context, "movie_votes").await;
    }

    let movies = sqlx::query_as!(
        Movie,
        "SELECT * FROM movies WHERE
        (guild_id = $1 AND nominated);",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_all(&context.pool)
    .await?;

    let movie: &Movie = movies
        .iter()
        .find(|m| m.title == content)
        .ok_or(anyhow!("Coudn't find movie in votable movies"))?;

    sqlx::query!(
        "INSERT INTO movie_votes (guild_id, member_id, id) VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, member_id, id) DO NOTHING;",
        context.message.guild_id.unwrap().to_string(),
        context.message.author.id.to_string(),
        movie.id
    )
    .execute(&context.pool)
    .await?;

    Ok(Response::Reaction("✅".into()))
}

pub async fn movie(context: &MessageContext) -> Result<Response> {
    let mut content = context.content.split(" ");
    match content.next() {
        Some("nominate") => nominate(&context, content.collect::<Vec<_>>().join(" ")).await,
        Some("set-url") => set_url(&context, content.collect::<Vec<_>>().join(" ")).await,
        Some("suggest") => suggestions_add(&context, content.collect::<Vec<_>>().join(" ")).await,
        Some("suggestions") => match content.next() {
            Some("add") => suggestions_add(&context, content.collect::<Vec<_>>().join(" ")).await,
            Some("list") => suggestions_list(&context).await,
            Some(&_) | None => suggestions_list(&context).await,
        },
        Some("vote") => vote(&context, content.collect::<Vec<_>>().join(" ")).await,
        Some(&_) | None => Ok(util::construct_response(
            context
                .http
                .create_message(context.message.channel_id)
                .content("unknown movie subcommand")
                .await,
        )),
    }
}
