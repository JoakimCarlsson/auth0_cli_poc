use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub id_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
}

pub async fn exchange_code_for_token(domain: &str, client_id: &str, client_secret: &str, redirect_uri: &str, code: &str) -> Result<(StatusCode, AuthResponse), Box<dyn Error>> {
    let client = Client::new();
    let res = client.post(format!("https://{}/oauth/token", domain))
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;
    let auth_response: AuthResponse = serde_json::from_str(&body)?;

    Ok((status, auth_response))
}