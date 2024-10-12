use std::sync::{Mutex};
use log::error;
use crate::http::{HttpContentType, HttpResult, Request, Response};
use crate::message::message_buffer::MessageBuffer;
use crate::route::method::Method;
use crate::route::route::Route;

const MAX_MESSAGE_SIZE: usize = 160*2 + 1; // 160 characters, 2 bytes per character to account for UTF-8

pub struct PostMessages {
    messages: &'static Mutex<MessageBuffer>
}

impl PostMessages {
    pub fn new(messages: &'static Mutex<MessageBuffer>) -> Self {
        PostMessages {
            messages
        }
    }
}

impl<'a> Route<'a> for PostMessages {
    fn route(&self) -> &'a str {
        "/messages"
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
                if data.len() > MAX_MESSAGE_SIZE {
                    return Response {
                        request,
                        status: HttpResult::InvalidRequest,
                        content_type: HttpContentType::Html,
                        data: String::from("Message too long")
                    };
                }

                messages.add_message(ip, data.clone());
            }
            None => {
                return Response {
                    request,
                    status: HttpResult::InvalidRequest,
                    content_type: HttpContentType::Html,
                    data: String::from("No message given")
                };
            }
        }

        Response {
            request,
            status: HttpResult::Ok,
            content_type: HttpContentType::Json,
            data: messages.cached()
        }
    }
}