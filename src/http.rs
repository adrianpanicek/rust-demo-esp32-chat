use crate::route::route::Route;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::sys::{httpd_req_to_sockfd, lwip_getpeername, lwip_inet_ntop, sockaddr, sockaddr_in6, socklen_t, EspError, AF_INET};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
#[allow(dead_code)] // For debug purposes
pub enum HttpServerError {
    FailedToConfigure(EspIOError),
    FailedToDefineRoute(EspError)
}

pub struct HttpServer<'a> {
    server: EspHttpServer<'a>
}

#[derive(Debug, Clone)]
pub enum HttpResult {
    Ok,
    InvalidRequest,
    InternalServerError
}

impl Into<u16> for HttpResult {
    fn into(self) -> u16 {
        match self {
            HttpResult::Ok => 200,
            HttpResult::InvalidRequest => 400,
            HttpResult::InternalServerError => 500
        }
    }
}

impl Display for HttpResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<u16>::into(self.clone()))
    }
}

#[derive(Copy, Clone)]
pub enum HttpContentType {
    Html,
    Javascript,
    Stylesheet,
    Json
}

impl Into<&str> for HttpContentType {
    fn into(self) -> &'static str {
        match self {
            HttpContentType::Html => "text/html",
            HttpContentType::Stylesheet => "text/css",
            HttpContentType::Javascript => "application/javascript",
            HttpContentType::Json => "application/json"
        }
    }
}

pub struct Request<'a, 'b> {
    req: esp_idf_svc::http::server::Request<&'a mut EspHttpConnection<'b>>,
    pub data: Option<String>,
}

pub struct Response<'a, 'b> {
    pub request: Request<'a, 'b>,
    pub status: HttpResult,
    pub content_type: HttpContentType,
    pub data: String
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub status: HttpResult,
    pub message: String
}

impl HttpError {
    pub fn new(status: HttpResult, message: String) -> Self {
        HttpError { status, message }
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP Error {}: {}", self.status, self.message)
    }
}


impl<'a, 'b> Request<'a,'b> {
    // Ugly way to get IP address from request as Rust bindings are not yet implemented, we have to access FFI
    pub fn ip(&mut self) -> Option<String> {
        unsafe {
            let conn = RawHandle::handle(self.req.connection());

            let sockfd = httpd_req_to_sockfd(conn);
            let mut addr: sockaddr_in6 = sockaddr_in6::default(); // ESP uses IPv6 addressing
            let typehack_addr = std::ptr::slice_from_raw_parts_mut(
                &mut addr as *mut _,
                size_of::<sockaddr_in6>()
            );
            let mut addr_size: socklen_t = size_of::<sockaddr_in6>() as socklen_t;
            let mut ipstr = [0u8; 4 * 4];

            match lwip_getpeername(sockfd, typehack_addr as *mut sockaddr, &mut addr_size) {
                0 => {
                    lwip_inet_ntop(
                        AF_INET as core::ffi::c_int,
                        &addr.sin6_addr.un.u32_addr[3] as *const core::ffi::c_uint as *mut core::ffi::c_void,
                        &mut ipstr as *mut _ as *mut core::ffi::c_char,
                        17
                    );

                    String::from_utf8(ipstr.to_vec()).map(|s| s.trim_end_matches('\0').to_string()).ok()
                },
                _ => {
                    None
                }
            }
        }
    }
}

impl<'a> HttpServer<'a> {
    fn process_request_data(req: &mut esp_idf_svc::http::server::Request<&mut EspHttpConnection>) -> Result<String, HttpError> {
        let mut buffer = [0u8; 4096]; // This basically means the largest request we can handle is 4KB

        req.read(&mut buffer)
            .map_err(|err| HttpError::new(HttpResult::InvalidRequest, format!("Could not read request data: {:?}", err)))
            .and_then(|len|
                String::from_utf8(buffer[0..len].to_vec())
                    .map_err(|err| HttpError::new(HttpResult::InvalidRequest, format!("Could not parse request data: {:?}", err)))
            )
    }

    pub fn add_route<T>(&mut self, route: T) -> Result<(), HttpServerError>
    where
        T: Route<'static> + Sync + Send + 'static
    {
        self.server.fn_handler(route.route(), route.method().clone().into(), move |mut req| {
            let request_data = Self::process_request_data(&mut req);

            let request = Request {
                req,
                data: request_data.ok(),
            };

            let result = route.handle(request);

            result.request.req.into_response(
                result.status.into(),
                None,
                vec![("Content-Type", result.content_type.into())].as_slice()
            ).and_then(|mut conn| conn.write(result.data.as_bytes()))
                .map(|_| ())
                .map_err(move |err| err.0)
        }).map_err(|err| {
            HttpServerError::FailedToDefineRoute(err)
        }).map(|_| ())
    }

    pub fn new() -> Result<Self, HttpServerError> {
        // Let's make it beefy
        EspHttpServer::new(&Configuration {
            max_open_sockets: 13,
            max_sessions: 256,
            stack_size: 4096*4,

            ..Default::default()
        }).or_else(|err| Err(HttpServerError::FailedToConfigure(err)))
            .and_then(|server| Ok(HttpServer { server }))
    }
}