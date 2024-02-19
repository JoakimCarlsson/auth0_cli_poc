mod cli;
mod server;
mod auth;

use cli::Cli;
use std::sync::{Arc, Mutex};
use clap::Parser;
use std::io::{self, Write};
use tokio;
use webbrowser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let code_received = Arc::new(Mutex::new(None::<String>));
    let redirect_uri = server::start_server(code_received.clone()).await;

    let auth_url = format!(
        "https://{}/authorize?client_id={}&response_type=code&redirect_uri={}&audience={}&scope=openid profile email&prompt=login",
        cli.auth0_domain, cli.auth0_client_id, redirect_uri, cli.audience
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
        },
        Err(e) => {
            println!("Failed to exchange code for token: {}", e);
        }
    }

    // Cleanup and exit
    println!("Press Enter to exit...");
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut String::new());
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