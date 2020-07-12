use anyhow::Result;
use async_trait::async_trait;

use crate::model::{MessageContext, Response};

mod avatar;
mod change_avatar;
mod choose;
mod invite;
mod movie;
mod owo;
mod ping;
mod shuffle;

pub use avatar::Avatar;
pub use change_avatar::ChangeAvatar;
pub use choose::Choose;
pub use invite::Invite;
pub use movie::Movie;
pub use owo::owo;
pub use ping::ping;
pub use shuffle::shuffle;

#[async_trait]
pub trait Command<T>
where
    T: Sized + Send + Sync,
{
    fn new(context: MessageContext) -> Self;

    async fn check(self: &Self) -> Result<bool> {
        Ok(true)
    }

    async fn execute(self: &mut Self) -> Result<Response>;
}

pub struct NoCommand(pub MessageContext);

#[async_trait]
impl Command<MessageContext> for NoCommand {
    fn new(context: MessageContext) -> Self {
        Self(context)
    }

    async fn execute(self: &mut Self) -> Result<Response> {
        Ok(Response::None)
    }
}
