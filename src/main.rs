mod cli;
mod server;
mod auth;

use cli::Cli;
use std::sync::{Arc, Mutex};
use clap::Parser;

extern crate clipboard;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use tokio;
use webbrowser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let code_received = Arc::new(Mutex::new(None::<String>));
    let redirect_uri = server::start_server(code_received.clone()).await;

    let auth_url = format!(
        "https://{}/authorize?client_id={}&response_type=code&redirect_uri={}&audience={}&scope={}&prompt=login",
        cli.auth0_domain, cli.auth0_client_id, redirect_uri, cli.audience, cli.auth0_scopes
    );

    if webbrowser::open(&auth_url).is_ok() {
        println!("Please log in via your web browser.");
    } else {
        println!("Please navigate to the following URL to log in: {}", auth_url);
    }

    let auth_code = wait_for_auth_code(code_received).await;

    let result = auth::exchange_code_for_token(
        &cli.auth0_domain,
        &cli.auth0_client_id,
        &cli.auth0_client_secret,
        &redirect_uri,
        &auth_code,
    )
        .await;

    match result {
        Ok((status, auth_response)) => {
            println!("HTTP Status: {}", status);
            println!("Access Token: {}", auth_response.access_token);

            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            if ctx.set_contents(auth_response.access_token.clone()).is_ok() {
                println!("Access token copied to clipboard.");
            } else {
                println!("Failed to copy access token to clipboard.");
            }
        },
        Err(e) => {
            println!("Failed to exchange code for token: {}", e);
        }
    }
}

async fn wait_for_auth_code(code_received: Arc<Mutex<Option<String>>>) -> String {
    loop {
        {
            let code_lock = code_received.lock().unwrap();
            if let Some(ref code) = *code_lock {
                return code.clone();
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}