#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub redis_skew_secs: i64,
    pub http_jwks_addr: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub jwt_kid: String,
    pub jwt_private_pem: String,
    pub jwt_public_pem: String,
    pub access_ttl: i64,
    pub refresh_ttl: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        Ok(Self {
            database_url: get(
                "DATABASE_URL",
                "postgres://auth_user:auth_pass@127.0.0.1:5432/auth_db",
            ),
            redis_url: get("REDIS_URL", "redis://127.0.0.1:6379"),
            redis_skew_secs: get("REDIS_SKEW_SECS", "120").parse().unwrap_or(120),
            http_jwks_addr: get("HTTP_JWKS_ADDR", "0.0.0.0:8080"),
            jwt_issuer: get("JWT_ISSUER", "auth.svc"),
            jwt_audience: get("JWT_AUDIENCE", "tax.api"),
            jwt_kid: get("JWT_KID", "dev-1"),
            jwt_private_pem: get("JWT_PRIVATE_PEM", "keys/dev_rsa_private.pem"),
            jwt_public_pem: get("JWT_PUBLIC_PEM", "keys/dev_rsa_public.pem"),
            access_ttl: get("ACCESS_TTL_SECS", "900").parse().unwrap_or(900),
            refresh_ttl: get("REFRESH_TTL_SECS", "2592000")
                .parse()
                .unwrap_or(2592000),
        })
    }
}
