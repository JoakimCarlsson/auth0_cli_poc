use reqwest::header::{CONTENT_TYPE};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use warp::Filter;
use tokio;
use webbrowser;
use reqwest::StatusCode;
use std::error::Error;

#[tokio::main]
async fn main() {
    let code_received = Arc::new(Mutex::new(None::<String>));

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

    let auth0_domain = "";
    let auth0_client_id = "";
    let redirect_uri = format!("http://{}:{}/callback", addr.ip(), addr.port());
    let audience = "";
    let auth_url = format!(
        "https://{}/authorize?client_id={}&response_type=code&redirect_uri={}&audience={}&scope=openid profile email&prompt=login",
        auth0_domain, auth0_client_id, redirect_uri, audience
    );

    if webbrowser::open(&auth_url).is_ok() {
        println!("Please log in via your web browser.");
    } else {
        println!("Please navigate to the following URL to log in: {}", auth_url);
    }

    let mut auth_code: Option<String> = None;
    while auth_code.is_none() {
        let code_lock = code_received.lock().unwrap();
        if code_lock.is_some() {
            auth_code = code_lock.clone();
        }
        drop(code_lock);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let auth0_client_secret = "";
    let result = exchange_code_for_token(&auth0_domain, &auth0_client_id, &auth0_client_secret, &redirect_uri, &auth_code.unwrap()).await;

    match result {
        Ok((status, auth_response)) => {
            println!("HTTP Status: {}", status);
            println!("Access Token: {}", auth_response.access_token);
        },
        Err(e) => {
            println!("Failed to exchange code for token: {}", e);
        }
    }
}

async fn exchange_code_for_token(domain: &str, client_id: &str, client_secret: &str, redirect_uri: &str, code: &str) -> Result<(StatusCode, AuthResponse), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client.post(format!("https://{}/oauth/token", domain))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri)
        ])
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;
    let auth_response: AuthResponse = serde_json::from_str(&body)?;

    Ok((status, auth_response))
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    id_token: String,
    expires_in: u64,
    token_type: String,
    scope: String,
}