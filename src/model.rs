use twilight::{
    http::{error::Error, Client as HttpClient},
    model::channel::Message,
};

#[derive(Debug)]
pub enum Response {
    Some(Message),
    Err(Error),
    None,
}

#[derive(Debug)]
pub struct Context {
    message: Message,
    http: HttpClient,
    content: String,
}
