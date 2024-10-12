
extern crate core;

mod wifi;
mod http;
mod message;
mod route;

use std::sync::{Mutex};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::{FreeRtos};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use log::{info};
use crate::http::{HttpContentType, HttpServer};
use crate::message::message_buffer::MessageBuffer;
use crate::route::get_messages::GetMessages;
use crate::route::route::{StaticRoute};
use crate::route::post_messages::{PostMessages};
use crate::wifi::Wifi;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

// This bakes the content of the files into the binary
static INDEX_HTML: &str = include_str!("../http/index.html");
static STYLE_CSS: &str = include_str!("../http/style.css");
static SCRIPT_JS: &str = include_str!("../http/script.js");

static STATIC_ROUTES: [(&str, HttpContentType, &str); 3] = [
    ("/", HttpContentType::Html, INDEX_HTML),
    ("/style.css", HttpContentType::Stylesheet, STYLE_CSS),
    ("/script.js", HttpContentType::Javascript, SCRIPT_JS)
];

#[link_section = ".dram1"] // This forces allocation to be in internal ram of ESP32
static MESSAGES: Mutex<MessageBuffer> = Mutex::new(MessageBuffer::new());

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Could not take peripherals");
    let sysloop = EspSystemEventLoop::take()
        .expect("Could not take Esp system event loop");
    let nvs = EspNvsPartition::<NvsDefault>::take()
        .expect("Could not take nvs partition");

    let mut wifi = match Wifi::new(peripherals.modem, sysloop, nvs) {
        Ok(wifi) => wifi,
        Err(err) => panic!("Could not initialize wifi: {:?}", err),
    };

    match wifi.connect(SSID, PASSWORD) {
        Ok(_) => info!("Connected to wifi"),
        Err(err) => panic!("Could not connect to wifi: {:?}", err),
    }

    let mut http = match HttpServer::new() {
        Ok(http) => http,
        Err(err) => panic!("Could not initialize http server: {:?}", err),
    };

    // Add routes for static content
    STATIC_ROUTES.iter().for_each(|(path, mime, content)| {
        http.add_route(StaticRoute::new(path, route::method::Method::GET, mime.clone(), content)).unwrap()
    });

    http.add_route(PostMessages::new(&MESSAGES)).unwrap();
    http.add_route(GetMessages::new(&MESSAGES)).unwrap();

    loop {
        FreeRtos::delay_ms(5u32);
    }
}
