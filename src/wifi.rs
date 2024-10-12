use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use crate::wifi::WifiError::FailedToInitialize;

#[derive(Debug)]
#[allow(dead_code)] // For debug purposes
pub enum WifiError {
    FailedToInitialize(EspError),
    FailedToScan(EspError),
    FailedToConfigure(EspError),
    FailedToConnect(EspError),
    SsidNotFound,
}

pub struct Wifi<'a> {
    wifi: BlockingWifi<EspWifi<'a>>,
}

impl<'a> Wifi<'a> {
    pub fn new(
        modem: Modem,
        sysloop: EspSystemEventLoop,
        nvs: EspNvsPartition<NvsDefault>
    ) -> Result<Self, WifiError> {
        let esp_wifi: EspWifi = EspWifi::new(modem, sysloop.clone(), Some(nvs)).unwrap();

        let configuration = Configuration::Client(ClientConfiguration::default());

        let mut wifi = match BlockingWifi::wrap(esp_wifi, sysloop) {
            Ok(wifi) => wifi,
            Err(inner) => return Err(WifiError::FailedToInitialize(inner)),
        };

        wifi.set_configuration(&configuration) // Configure wifi module
            .and(wifi.start())
            .map_err(|err| FailedToInitialize(err))// Start the wifi module
            .and_then(|_| Ok(Wifi {
                wifi,
            }))
    }

    fn configure(ssid: &str, password: &str, channel: u8) -> Configuration {
        Configuration::Client(
            ClientConfiguration {
                ssid: ssid.try_into().unwrap(),
                password: password.try_into().unwrap(),
                channel: Some(channel),
                ..Default::default()
            }
        )
    }

    pub fn connect(&mut self, ssid: &str, password: &str) -> Result<(), WifiError> {
        let scan_result = match self.wifi.scan() {
            Ok(aps) => aps,
            Err(err) => return Err(WifiError::FailedToScan(err)),
        };

        // Bit ugly but the esp-idf wrap is messy
        scan_result.into_iter().find(|ap| ap.ssid == ssid).ok_or(WifiError::SsidNotFound)
            .and_then(|ap| self.wifi.set_configuration(&Self::configure(ssid, password, ap.channel))
                .or_else(|err| Err(WifiError::FailedToConfigure(err))))
            .and(self.wifi.connect().or_else(|err| Err(WifiError::FailedToConnect(err))))
            .and(self.wifi.wait_netif_up().or_else(|err| Err(WifiError::FailedToConnect(err))))
    }
}