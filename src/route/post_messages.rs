use std::sync::{Mutex};
use log::error;
use crate::http::{HttpContentType, HttpResult, Request, Response};
use crate::message::message_buffer::MessageBuffer;
use crate::route::method::Method;
use crate::route::route::Route;

pub struct PostMessage {
    messages: &'static Mutex<MessageBuffer>
}

impl PostMessage {
    pub fn new(messages: &'static Mutex<MessageBuffer>) -> Self {
        PostMessage {
            messages
        }
    }
}

impl<'a> Route<'a> for PostMessage {
    fn route(&self) -> &'a str {
        "/message"
    }

    fn method(&self) -> &Method {
        &Method::POST
    }

    fn handle<'c, 'd>(&self, mut request: Request<'c, 'd>) -> Response<'c, 'd> {
        // Thanks to shadowing, we can name the buffer the same as the buffer in the outer scope
        let messages = &self.messages;
        let mut messages = match (*messages).lock() {
            Ok(mb) => mb,
            Err(err) => {
                error!("Could not lock message buffer: {:?}", err);
                return Response {
                    request,
                    status: HttpResult::InternalServerError,
                    content_type: HttpContentType::Html,
                    data: String::from("Could not read messages")
                };
            }
        };

        let ip = request.ip().unwrap_or("Unknown IP".to_string());
        match request.data {
            Some(ref data) => {
                messages.add_message(ip, data.clone());
            }
            None => {}
        }

        Response {
            request,
            status: HttpResult::Ok,
            content_type: HttpContentType::Json,
            data: messages.cached()
        }
    }
}