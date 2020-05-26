use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table_if_not_exists("guilds", |table| {
        table.add_column("id",   types::text().unique(true));
        table.add_column("name", types::text());
    });

    m.create_table_if_not_exists("settings", |table| {
        table.add_column("guild_id",             types::text().unique(true).nullable(false));
        table.add_column("starboard_channel_id", types::text());
        table.add_column("starboard_emoji",      types::text().default("⭐"));
        table.add_column("starboard_min_stars",  types::integer().default(1));
        table.add_column("movies_repeat_every",  types::integer().default(7));
    });

    m.create_table_if_not_exists("invite_roles", |table| {
        table.add_column("guild_id",    types::text().nullable(false));
        table.add_column("role_id",     types::text().nullable(false));
        table.add_column("invite_code", types::text().nullable(false));
    });

    m.create_table_if_not_exists("roleme_roles", |table| {
        table.add_column("guild_id",    types::text().nullable(false));
        table.add_column("role_id",     types::text().nullable(false));
        table.add_column("invite_code", types::text().nullable(false));
    });

    m.create_table_if_not_exists("starboard", |table| {
        table.add_column("guild_id",   types::text().nullable(false));
        table.add_column("member_id",  types::text().nullable(false));
        table.add_column("channel_id", types::text().nullable(false));
        table.add_column("message_id", types::text().nullable(false));
        table.add_column("post_id",    types::text().nullable(false));
        table.add_column("star_count", types::integer().nullable(false));
        table.add_column("date",       types::date().nullable(false));
    });

    m.create_table_if_not_exists("movies", |table| {
        table.add_column("guild_id",   types::text().nullable(false));
        table.add_column("member_id",  types::text().nullable(false));
        table.add_column("id",         types::integer().increments(true).unique(true));
        table.add_column("title",      types::text().nullable(false));
        table.add_column("url",        types::text());
        table.add_column("watch_date", types::date());
        table.add_column("nominated",  types::boolean());
    });

    m.create_table_if_not_exists("movie_votes", |table| {
        table.add_column("guild_id",  types::text().nullable(false));
        table.add_column("member_id", types::text().nullable(false));
        table.add_column("id",        types::integer());
    });

    m.create_table_if_not_exists("movie_dates", |table| {
        table.add_column("guild_id",   types::text().nullable(false));
        table.add_column("watch_date", types::date());
        table.add_column("id",         types::integer());
    });

    m.make::<Pg>()
}
