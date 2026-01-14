use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_SUPABASE_URL: &str = "https://your-project.supabase.co";
const DEFAULT_SUPABASE_ANON_KEY: &str = "your_anon_key";

pub(crate) fn default_supabase_url() -> &'static str {
    DEFAULT_SUPABASE_URL
}

pub(crate) fn default_supabase_anon_key() -> &'static str {
    DEFAULT_SUPABASE_ANON_KEY
}

pub(crate) fn login(
    email: &str,
    password: &str,
    supabase_url: &str,
    supabase_anon_key: &str,
) -> Result<LoginResponse> {
    let url = format!(
        "{}/auth/v1/token?grant_type=password",
        supabase_url.trim_end_matches('/')
    );

    let client = Client::new();
    let response = client
        .post(url)
        .header("apikey", supabase_anon_key)
        .json(&LoginRequest { email, password })
        .send()?
        .error_for_status()?;

    Ok(response.json()?)
}

#[derive(Deserialize)]
pub(crate) struct LoginResponse {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) expires_in: i64,
    pub(crate) user: LoginUser,
}

#[derive(Deserialize)]
pub(crate) struct LoginUser {
    pub(crate) id: String,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}
