use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
    pub max_size: u32,
    pub skew_secs: i64,
}

#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub leeway_secs: u64,
    pub access_ttl_secs: i64,
}

#[derive(Clone, Debug)]
pub struct RefreshConfig {
    pub ttl_secs: i64,
    pub prefix: &'static str,
}

// #[derive(Clone, Debug)]
// pub struct SesConfig {
//     pub source: String,
//     pub configuration_set: Option<String>,
// }

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub port: Option<u16>,
    pub timeout_secs: u64,
    pub max_retries: u8,
}

#[derive(Clone, Debug)]
pub struct PasswordConfig {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_lanes: u32,
    pub pepper: String,
}

#[derive(Clone, Debug)]
pub struct VerifyEmailConfig {
    pub base_url: String,
    pub token_ttl_secs: i64,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DbConfig,
    pub cache: RedisConfig,
    pub jwt: JwtConfig,
    pub refresh: RefreshConfig,
    pub mailer: SmtpConfig,
    pub password: PasswordConfig,
    pub verify: VerifyEmailConfig,
    pub dummy_password_hash: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());

        let server = ServerConfig {
            addr: get("APP_ADDR", "0.0.0.0:8085"),
        };

        let db = DbConfig {
            url: get("DATABASE_URL", "postgres://auth_user:2803@127.0.0.1:5433/auth"),
            max_connections: get("DB_MAX_CONNS", "10").parse().unwrap_or(10),
            timeout: get("DB_CONN_TIMEOUT", "5").parse().unwrap_or(5),
        };

        let cache = RedisConfig {
            url: get("REDIS_URL", "redis://127.0.0.1:6379"),
            max_size: get("REDIS_MAX_CONNS", "4").parse().unwrap_or(4),
            skew_secs: get("REDIS_SKEW_SECS", "120").parse().unwrap_or(120),
        };

        let jwt = JwtConfig {
            issuer: get("JWT_ISSUER", "auth.svc"),
            audience: get("JWT_AUDIENCE", "tax.api"),
            leeway_secs: get("JWT_LEEWAY_SECS", "5").parse().unwrap_or(5),
            access_ttl_secs: get("ACCESS_TTL_SECS", "900").parse().unwrap_or(900),
        };

        let refresh = RefreshConfig {
            ttl_secs: get("REFRESH_TTL_SECS", "2592000").parse().unwrap_or(2_592_000),
            prefix: "r1.",
        };

        // let ses = SesConfig {
        //     source: get("AWS_SES_SOURCE", "no-reply@example.com"),
        //     configuration_set: std::env::var("AWS_SES_CONFIG_SET").ok(),
        // };
        let mailer = SmtpConfig {
            host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.yandex.ru".into()),
            port: std::env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()),
            username: std::env::var("EMAIL").expect("EMAIL not set"),
            password: std::env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD not set"),
            display_name: std::env::var("EMAIL_NAME").unwrap_or_else(|_| "NoReply".into()),
            timeout_secs: std::env::var("SMTP_TIMEOUT").ok().and_then(|s| s.parse().ok()).unwrap_or(5),
            max_retries: std::env::var("SMTP_MAX_RETRIES").ok().and_then(|s| s.parse().ok()).unwrap_or(3),
        };

        let password = PasswordConfig {
            m_cost_kib: get("KDF_M_COST_KIB", "65536").parse().unwrap_or(65536),
            t_cost: get("KDF_T_COST", "3").parse().unwrap_or(3),
            p_lanes: get("KDF_P_LANES", "1").parse().unwrap_or(1),
            pepper: std::env::var("PASSWORD_PEPPER").with_context(|| "PASSWORD_PEPPER must be set (base64)")?,
        };

        let verify = VerifyEmailConfig {
            base_url: get("VERIFY_BASE_URL", "http://localhost:8085/auth/verify?token="),
            token_ttl_secs: get("EMAIL_VERIFY_TTL_SECS", "86400").parse().unwrap_or(86_400),
        };

        let dummy_password_hash = get(
            "DUMMY_PASSWORD_HASH",
            "$argon2id$v=19$m=65536,t=3,p=1$R0VORVJBVEVEX1NBTFQ$8v0QWnN8S2sRzR2VdX1lA4O3p2y1W8Q4G8g7w8r2s1U",
        );

        Ok(Self {
            server,
            db,
            cache,
            jwt,
            refresh,
            mailer,
            password,
            verify,
            dummy_password_hash,
        })
    }
}
