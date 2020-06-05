use anyhow::Result;

use crate::model::{MessageContext, ReactionContext, Response};

mod movie_votes;

/// Create a reaction menu of a specified type.
///
/// The format of the redis key for reaction menu state is as follows:
///
/// `reaction_menu:{channel_id}:{message_id}:{menu_type}`
///
/// where menu_type is one of `movie_votes`.
pub async fn create_menu(context: &MessageContext, menu_type: &str) -> Result<Response> {
    match menu_type {
        "movie_votes" => movie_votes::create_menu(&context).await,
        _ => Ok(Response::None),
    }
}

pub async fn handle_event(context: &ReactionContext, menu_type: &str) -> Result<()> {
    match menu_type {
        "movie_votes" => movie_votes::handle_event(&context).await,
        _ => Ok(()),
    }
}
