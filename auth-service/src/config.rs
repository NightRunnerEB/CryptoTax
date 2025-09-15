use anyhow::{Context, Result};
use cache::RedisCacheConfig;

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub addr: String,
}

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
}

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
    pub max_size: u32,
    pub skew_secs: i64,
}

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub leeway_secs: u64,
    pub access_ttl_secs: i64,
}

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct RefreshConfig {
    pub ttl_secs: i64,
    pub prefix: &'static str,
}

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct SesConfig {
    pub source: String,
    pub configuration_set: Option<String>,
}

// СДЕЛАЛ
#[derive(Clone, Debug)]
pub struct KdfConfig {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_lanes: u32,
    pub pepper_current_b64: String,
}

// СДЕЛАЛ
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
    pub ses: SesConfig,
    pub kdf: KdfConfig,
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

        let ses = SesConfig {
            source: get("AWS_SES_SOURCE", "no-reply@example.com"),
            configuration_set: None,
        };

        let kdf = KdfConfig {
            m_cost_kib: get("KDF_M_COST_KIB", "65536").parse().unwrap_or(65536),
            t_cost: get("KDF_T_COST", "3").parse().unwrap_or(3),
            p_lanes: get("KDF_P_LANES", "1").parse().unwrap_or(1),
            pepper_current_b64: std::env::var("PEPPER_CURRENT_B64")
                .with_context(|| "PEPPER_CURRENT_B64 must be set (base64)")?,
        };

        let verify = VerifyEmailConfig {
            base_url: get("VERIFY_BASE_URL", "https://example.com/verify?token="),
            token_ttl_secs: get("VERIFY_EMAIL_TTL_SECS", "86400").parse().unwrap_or(86_400),
        };

        // ???
        let dummy_password_hash = get(
            "DUMMY_PASSWORD_HASH",
            // Вставь валидный PHC под argon2id
            "$argon2id$v=19$m=65536,t=3,p=1$R0VORVJBVEVEX1NBTFQ$8v0QWnN8S2sRzR2VdX1lA4O3p2y1W8Q4G8g7w8r2s1U",
        );

        Ok(Self {
            server,
            db,
            cache,
            jwt,
            refresh,
            ses,
            kdf,
            verify,
            dummy_password_hash,
        })
    }
}
