use dbus::blocking::Connection;
use dbus::arg::Variant;
use std::{collections::HashMap, time::Duration};


pub fn open_uri(uri: &str) -> Result<(), Box<dyn std::error::Error>> {
    // std::env::set_var("AURORA_APP_ID", "com.lmaxyz.Haboost");
    // std::env::set_var("AURORA_TASK_ID", "com.lmaxyz.Haboost");


    let conn = Connection::new_session()?;

    // Second, create a wrapper struct around the connection that makes it easy
    // to send method calls to a specific destination and path.
    let proxy = conn.with_proxy("ru.omp.RuntimeManager", "/ru/omp/RuntimeManager/Intents1", Duration::from_millis(5000));

    let hints: HashMap<String, Variant<String>> = HashMap::new();
    let mut data: HashMap<String, Variant<String>> = HashMap::new();
    data.insert("uri".to_string(), Variant(uri.to_string()));

    let (_,): (String,) = proxy.method_call("ru.omp.RuntimeManager.Intents1", "InvokeIntent", ("OpenURI", hints, data)).unwrap_or((String::default(),));
    // for name in names.iter() {
    //     if !name.starts_with(":") && name.contains("RuntimeManager") {
    //         let proxy = conn.with_proxy(name, format!("/{}/Intents1", name.split('.').map(|i| format!("{i}/")).collect::<String>().strip_suffix("/").unwrap()), Duration::from_millis(5000));
    //         let introspect = proxy.introspect().unwrap_or(String::new());
    //         println!("{introspect}");
    //         // if methods.len() > 0 {
    //         //     for method in methods { println!("\t{}", method); }
    //         // }
    //     }
    // }

    // Let's print all the names to stdout.
    // for name in names { println!("{}", name); }
    // println!();
    Ok(())
}
