#[derive(Debug)]
pub enum SettingRole {
    Movies,
}

impl Display for SettingRole {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Movies => write!(f, "movies"),
        }
    }
}

#[derive(Debug)]
pub struct ReactionContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub hyper: HyperClient<HttpsConnector<HttpConnector>>,
    pub postgres: PgPool,
    pub redis: RedisPool,
    pub reaction: Box<ReactionAdd>,
}

impl ReactionContext {
    pub fn new(context: BaseContext, reaction: Box<ReactionAdd>) -> Self {
        Self {
            cache: context.cache,
            http: context.http,
            hyper: context.hyper,
            postgres: context.postgres,
            redis: context.redis,
            reaction,
        }
    }
}
