#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
}

impl Into<esp_idf_svc::http::Method> for Method {
    fn into(self) -> esp_idf_svc::http::Method {
        match self {
            Method::GET => esp_idf_svc::http::Method::Get,
            Method::POST => esp_idf_svc::http::Method::Post,
        }
    }
}