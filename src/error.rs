use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Invalid Token: {}", source))]
    SerenityClientInvalidToken { source: serenity::Error },
    #[snafu(display("Failed to boot shard: {}", source))]
    SerenityClientShardBootFailure { source: serenity::Error },

    #[snafu(display("Message send error: {}", source))]
    SerenityMessageSendError { source: serenity::Error },
}
