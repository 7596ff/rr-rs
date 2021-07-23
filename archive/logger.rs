pub fn reaction_error(context: &ReactionContext, why: &anyhow::Error, command: String) {
    error!(
        "channel:{} command:{}\nerror processing reaction\n{:?}",
        context.reaction.channel_id, command, why
    );
}
