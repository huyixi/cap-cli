use anyhow::Result;
use std::env;

use crate::db::{Db, set_kv};

mod supabase;

pub(crate) fn login(db: &Db, email: &str, password: &str) -> Result<()> {
    let supabase_url =
        env::var("SUPABASE_URL").unwrap_or_else(|_| supabase::default_supabase_url().to_string());
    let supabase_anon_key = env::var("SUPABASE_ANON_KEY")
        .unwrap_or_else(|_| supabase::default_supabase_anon_key().to_string());

    let login_response = supabase::login(email, password, &supabase_url, &supabase_anon_key)?;
    set_kv(db, "auth_access_token", &login_response.access_token)?;
    set_kv(db, "auth_refresh_token", &login_response.refresh_token)?;
    set_kv(
        db,
        "auth_expires_in",
        &login_response.expires_in.to_string(),
    )?;
    set_kv(db, "auth_user_id", &login_response.user.id)?;
    println!("Logged in as {}", login_response.user.id);
    Ok(())
}
