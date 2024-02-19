use std::sync::{Arc, Mutex};
use warp::Filter;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}

pub async fn start_server(code_received: Arc<Mutex<Option<String>>>) -> String {
    let code_received_filter = {
        let code_received = code_received.clone();
        warp::any().map(move || code_received.clone())
    };

    let server = warp::path("callback")
        .and(warp::query::<CallbackQuery>())
        .and(code_received_filter)
        .map(|query: CallbackQuery, code_received: Arc<Mutex<Option<String>>>| {
            let mut code_lock = code_received.lock().unwrap();
            *code_lock = Some(query.code.clone());
            warp::reply::html("Authentication successful! You can close this window.")
        });

    let (addr, server) = warp::serve(server)
        .bind_ephemeral(([127, 0, 0, 1], 3000));

    tokio::spawn(server);

    format!("http://{}:{}/callback", addr.ip(), addr.port())
}