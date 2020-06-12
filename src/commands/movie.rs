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

    let response = match nominated.nominated {
        true => format!("✅ {} is nominated", movie.title),
        false => format!("✅ {} is **no longer** nominated", movie.title),
    };

    let reply = context.reply(response).await;
    Ok(util::construct_response(reply))
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
        "UPDATE movies SET url = $1 WHERE
        (guild_id = $2 AND title = $3);",
        url,
        context.message.guild_id.unwrap().to_string(),
        title
    )
    .execute(&context.pool)
    .await?;

    let reaction = context.react("✅").await;
    Ok(Response::Reaction(reaction))
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

    let reaction = context.react("✅").await;
    Ok(Response::Reaction(reaction))
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

    let mut content: String = format!(
        "List of suggestions by **{}**\n",
        context.message.author.name
    )
    .into();

    for movie in movies {
        if movie.nominated {
            content.push_str(&format!("- **{}** (nominated)\n", movie.title));
        } else {
            content.push_str(&format!("- {}\n", movie.title));
        }
    }

    content.push_str("Nominate a movie for voting with `katze movie nominate <name>`.");

    let reply = context.reply(content).await;
    Ok(util::construct_response(reply))
}

async fn vote(context: &MessageContext, content: String) -> Result<Response> {
    if content.len() < 1 {
        return reactions::create_menu(&context, "movie_votes").await;
    }

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

    let reaction = context.react("✅").await;
    Ok(Response::Reaction(reaction))
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
                .content("unknown movie subcommand")?
                .await,
        )),
    }
}
