use dbus::arg::Variant;
use dbus::blocking::Connection;
use std::{collections::HashMap, time::Duration};

pub fn open_uri(uri: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "ru.omp.RuntimeManager",
        "/ru/omp/RuntimeManager/Intents1",
        Duration::from_millis(500),
    );

    let hints: HashMap<String, Variant<String>> = HashMap::new();
    let mut data: HashMap<String, Variant<String>> = HashMap::new();
    data.insert("uri".to_string(), Variant(uri.to_string()));

    match proxy.method_call(
        "ru.omp.RuntimeManager.Intents1",
        "InvokeIntent",
        ("OpenURI", hints, data),
    ) {
        Ok(res) => {
            let _: (String,) = res;
        }
        Err(e) => {
            log::error!("Error with URI opening: {}", e)
        }
    }

    Ok(())
}
