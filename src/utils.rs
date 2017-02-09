// Various helper functions

use time;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

pub fn get_client() -> Client {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    Client::with_connector(connector)
}

pub fn get_current_time() -> String {
    format!("{}", time::now_utc().rfc3339()).to_string()
}
