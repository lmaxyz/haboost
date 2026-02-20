use dbus::arg::Variant;
use dbus::blocking::Connection;
use std::{collections::HashMap, time::Duration};

pub fn open_uri<F: FnOnce(String) + Send + 'static>(uri: &str, success_cb: F) {
    let uri = uri.to_string();
    std::thread::spawn(|| {
        let conn = Connection::new_session().unwrap();
        let proxy = conn.with_proxy(
            "ru.omp.RuntimeManager",
            "/ru/omp/RuntimeManager/Intents1",
            Duration::from_millis(5000),
        );

        let hints: HashMap<String, Variant<String>> = HashMap::new();
        let mut data: HashMap<String, Variant<String>> = HashMap::new();
        data.insert("uri".to_string(), Variant(uri));
        match proxy.method_call(
            "ru.omp.RuntimeManager.Intents1",
            "InvokeIntent",
            ("OpenURI", hints, data),
        ) {
            Ok(res) => {
                let response: (Vec<u8>,) = res;
                let resp_str = String::from_utf8_lossy(&response.0);
                success_cb(resp_str.to_string());
            }
            Err(_e) => {
                // Ошибки при открытии URL нет смысла обрабатывать
                // log::error!("Error with URI opening: {}", e)
            }
        }
    });
}
