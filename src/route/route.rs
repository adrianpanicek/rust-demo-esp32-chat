use crate::http::{HttpContentType, HttpResult, Request, Response};
use crate::route::method::Method;

pub trait Route<'a> {
    fn route(&self) -> &'a str;
    fn method(&self) -> &Method;
    fn handle<'c, 'd>(&self, request: Request<'c, 'd>) -> Response<'c, 'd>;
}

pub struct StaticRoute<'a> {
    route: &'a str,
    method: Method,
    content_type: HttpContentType,
    content: &'a str,
}

impl<'a> StaticRoute<'a> {
    pub fn new(route: &'a str, method: Method, content_type: HttpContentType, content: &'a str) -> Self {
        StaticRoute {
            route,
            method,
            content_type,
            content,
        }
    }
}

impl<'a> Route<'a> for StaticRoute<'a> {
    fn route(&self) -> &'a str {
        self.route
    }

    fn method(&self) -> &Method {
        &self.method
    }

    fn handle<'c, 'd>(&self, request: Request<'c, 'd>) -> Response<'c, 'd> {
        Response {
            request,
            status: HttpResult::Ok,
            content_type: self.content_type,
            data: self.content.to_string(),
        }
    }
}