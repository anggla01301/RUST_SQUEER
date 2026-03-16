use std::env;

fn cookie_secure() -> bool {
    env::var("COOKIE_SECURE")
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(true)
}

fn cookie_same_site() -> &'static str {
    match env::var("COOKIE_SAME_SITE")
        .unwrap_or_else(|_| "Strict".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "lax" => "Lax",
        "none" => "None",
        _ => "Strict",
    }
}

fn refresh_cookie_suffix(max_age: i64) -> String {
    let secure = if cookie_secure() { "; Secure" } else { "" };
    format!(
        "; HttpOnly; Path=/; Max-Age={max_age}; SameSite={}{}",
        cookie_same_site(),
        secure
    )
}

pub fn build_refresh_cookie(refresh_token: &str, max_age: i64) -> String {
    format!(
        "refreshToken={}{}",
        refresh_token,
        refresh_cookie_suffix(max_age)
    )
}

pub fn clear_refresh_cookie() -> String {
    format!("refreshToken={}", refresh_cookie_suffix(0))
}
