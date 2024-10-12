use std::sync::{Mutex};
use log::error;
use crate::http::{HttpContentType, HttpResult, Request, Response};
use crate::message::message_buffer::MessageBuffer;
use crate::route::method::Method;
use crate::route::route::Route;

pub struct GetMessages {
    messages: &'static Mutex<MessageBuffer>
}

impl GetMessages {
    pub fn new(messages: &'static Mutex<MessageBuffer>) -> Self {
        GetMessages {
            messages
        }
    }
}

impl<'a> Route<'a> for GetMessages {
    fn route(&self) -> &'a str {
        "/messages"
    }

    fn method(&self) -> &Method {
        &Method::GET
    }

    fn handle<'c, 'd>(&self, request: Request<'c, 'd>) -> Response<'c, 'd> {
        let messages = match (*&self.messages).lock() {
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

        Response {
            request,
            status: HttpResult::Ok,
            content_type: HttpContentType::Json,
            data: messages.cached()
        }
    }
}