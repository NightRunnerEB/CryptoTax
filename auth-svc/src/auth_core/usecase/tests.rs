use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use axum::async_trait;
use chrono::{Duration, Utc};
use uuid::Uuid;

use super::{AuthService, AuthUseCases};
use crate::{
    auth_core::{
        errors::{AuthError, Result},
        models::*,
        ports::*,
        services::*,
    },
    config::VerifyEmailConfig,
};

type TestAuthUseCases = AuthUseCases<
    UserRepoStub,
    SessionRepoStub,
    RefreshRepoStub,
    PasswordHasherStub,
    AccessTokenIssuerStub,
    RefreshTokenFactoryStub,
    RevocationCacheStub,
    EmailVerificationRepoStub,
    MailerStub,
    TaxProfileClientStub,
>;

fn take_expected<T>(slot: &Mutex<Option<T>>, name: &str) -> T {
    slot.lock().expect("mutex poisoned").take().unwrap_or_else(|| panic!("unexpected call: {name}"))
}

#[derive(Default)]
struct UserRepoStub {
    create_user_res: Mutex<Option<Result<Option<Uid>>>>,
    find_by_email_res: Mutex<Option<Result<Option<UserWithHash>>>>,
    delete_pending_user_res: Mutex<Option<Result<bool>>>,
    delete_pending_user_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl UserRepo for UserRepoStub {
    async fn create_user(&self, _email: &str, _password_hash: &str) -> Result<Option<Uid>> {
        take_expected(&self.create_user_res, "UserRepo.create_user")
    }

    async fn find_by_email(&self, _email_lower: &str) -> Result<Option<UserWithHash>> {
        take_expected(&self.find_by_email_res, "UserRepo.find_by_email")
    }

    async fn activate(&self, _user_id: Uid) -> Result<bool> {
        panic!("unexpected call: UserRepo.activate");
    }

    async fn update_password(&self, _user_id: Uid, _password_hash: &str) -> Result<()> {
        panic!("unexpected call: UserRepo.update_password");
    }

    async fn delete_pending_user(&self, _user_id: Uid) -> Result<bool> {
        self.delete_pending_user_calls.fetch_add(1, Ordering::Relaxed);
        take_expected(&self.delete_pending_user_res, "UserRepo.delete_pending_user")
    }
}

#[derive(Default)]
struct SessionRepoStub {
    set_status_res: Mutex<Option<Result<()>>>,
    set_status_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SessionRepo for SessionRepoStub {
    async fn create(&self, _user_id: Uid, _ip: Option<String>, _ua: Option<String>) -> Result<Session> {
        panic!("unexpected call: SessionRepo.create");
    }

    async fn get(&self, _session_id: Uid) -> Result<Option<Session>> {
        panic!("unexpected call: SessionRepo.get");
    }

    async fn set_status(&self, _session_id: Uid, _status: SessionStatus) -> Result<()> {
        self.set_status_calls.fetch_add(1, Ordering::Relaxed);
        take_expected(&self.set_status_res, "SessionRepo.set_status")
    }

    async fn touch(&self, _session_id: Uid) -> Result<()> {
        panic!("unexpected call: SessionRepo.touch");
    }

    async fn list_for_user(&self, _user_id: Uid) -> Result<Vec<Session>> {
        panic!("unexpected call: SessionRepo.list_for_user");
    }
}

#[derive(Default)]
struct RefreshRepoStub {
    get_by_hash_res: Mutex<Option<Result<Option<RefreshToken>>>>,
    revoke_all_for_session_res: Mutex<Option<Result<()>>>,
    revoke_all_for_session_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl RefreshRepo for RefreshRepoStub {
    async fn get_by_hash(&self, _hash: &[u8]) -> Result<Option<RefreshToken>> {
        take_expected(&self.get_by_hash_res, "RefreshRepo.get_by_hash")
    }

    async fn mark_rotated(&self, _jti: Uid) -> Result<bool> {
        panic!("unexpected call: RefreshRepo.mark_rotated");
    }

    async fn insert(&self, _rec: NewRefresh) -> Result<()> {
        panic!("unexpected call: RefreshRepo.insert");
    }

    async fn revoke_all_for_session(&self, _session_id: Uid) -> Result<()> {
        self.revoke_all_for_session_calls.fetch_add(1, Ordering::Relaxed);
        take_expected(&self.revoke_all_for_session_res, "RefreshRepo.revoke_all_for_session")
    }
}

#[derive(Default)]
struct PasswordHasherStub {
    hash_res: Mutex<Option<Result<String>>>,
    verify_res: Mutex<Option<Result<bool>>>,
}

impl PasswordHasher for PasswordHasherStub {
    fn hash(&self, _plain: &str) -> Result<String> {
        take_expected(&self.hash_res, "PasswordHasher.hash")
    }

    fn verify(&self, _hash: &str, _plain: &str) -> Result<bool> {
        take_expected(&self.verify_res, "PasswordHasher.verify")
    }
}

#[derive(Default)]
struct AccessTokenIssuerStub;

impl AccessTokenIssuer for AccessTokenIssuerStub {
    fn issue_token(&self, _user_id: Uid, _session_id: Uid, _roles: &[String]) -> Result<SignedToken> {
        panic!("unexpected call: AccessTokenIssuer.issue_token");
    }

    fn validate(&self, _token: &str) -> Result<AccessClaims> {
        panic!("unexpected call: AccessTokenIssuer.validate");
    }
}

#[derive(Default)]
struct RefreshTokenFactoryStub {
    hash_res: Mutex<Option<Vec<u8>>>,
}

impl RefreshTokenFactory for RefreshTokenFactoryStub {
    fn new_pair(&self) -> RefreshPair {
        panic!("unexpected call: RefreshTokenFactory.new_pair");
    }

    fn hash(&self, _token_plain: &str) -> Vec<u8> {
        take_expected(&self.hash_res, "RefreshTokenFactory.hash")
    }
}

#[derive(Default)]
struct RevocationCacheStub {
    check_refresh_res: Mutex<Option<Result<Option<RefreshBlockReason>>>>,
    revoke_all_for_session_res: Mutex<Option<Result<()>>>,
    revoke_all_for_session_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl RevocationCache for RevocationCacheStub {
    async fn check_refresh(&self, _session_id: Uid, _token_hash_b64: &str) -> Result<Option<RefreshBlockReason>> {
        take_expected(&self.check_refresh_res, "RevocationCache.check_refresh")
    }

    async fn mark_refresh_rotated(&self, _token_hash_b64: &str, _seconds_left: i64) -> Result<()> {
        panic!("unexpected call: RevocationCache.mark_refresh_rotated");
    }

    async fn revoke_all_for_session(&self, _session_id: Uid, _session_ttl_secs: i64) -> Result<()> {
        self.revoke_all_for_session_calls.fetch_add(1, Ordering::Relaxed);
        take_expected(&self.revoke_all_for_session_res, "RevocationCache.revoke_all_for_session")
    }
}

#[derive(Default)]
struct EmailVerificationRepoStub {
    create_token_res: Mutex<Option<Result<()>>>,
    create_token_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl EmailVerificationRepo for EmailVerificationRepoStub {
    async fn create_token(
        &self, _user_id: Uid, _token_hash: Vec<u8>, _sent_to: &str, _expires_at: chrono::DateTime<Utc>,
    ) -> Result<()> {
        self.create_token_calls.fetch_add(1, Ordering::Relaxed);
        take_expected(&self.create_token_res, "EmailVerificationRepo.create_token")
    }

    async fn revoke_all_for_user(&self, _user_id: Uid) -> Result<()> {
        panic!("unexpected call: EmailVerificationRepo.revoke_all_for_user");
    }

    async fn consume_by_hash(&self, _token_hash: &[u8]) -> Result<Option<Uid>> {
        panic!("unexpected call: EmailVerificationRepo.consume_by_hash");
    }
}

#[derive(Default)]
struct MailerStub {
    send_verification_res: Mutex<Option<Result<()>>>,
    send_verification_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Mailer for MailerStub {
    async fn send_verification(&self, _to: &str, _verify_link: &str) -> Result<()> {
        self.send_verification_calls.fetch_add(1, Ordering::Relaxed);
        take_expected(&self.send_verification_res, "Mailer.send_verification")
    }
}

#[derive(Default)]
struct TaxProfileClientStub {
    upsert_res: Mutex<Option<Result<()>>>,
    upsert_calls: Arc<AtomicUsize>,
    last_upsert_user_id: Mutex<Option<Uid>>,
}

#[async_trait]
impl TaxProfileClient for TaxProfileClientStub {
    async fn upsert_tax_profile(&self, user_id: Uid, _profile: &RegisterTaxProfile) -> Result<()> {
        self.upsert_calls.fetch_add(1, Ordering::Relaxed);
        *self.last_upsert_user_id.lock().expect("mutex poisoned") = Some(user_id);
        take_expected(&self.upsert_res, "TaxProfileClient.upsert_tax_profile")
    }
}

fn dummy_tax_profile() -> RegisterTaxProfile {
    RegisterTaxProfile {
        inn: "123456789012".to_string(),
        last_name: "Ivanov".to_string(),
        first_name: "Ivan".to_string(),
        middle_name: "".to_string(),
        jurisdiction: "RU".to_string(),
        timezone: "Europe/Moscow".to_string(),
        phone: "".to_string(),
        wallets: vec![],
        tax_residency_status: "resident".to_string(),
        taxpayer_type: "individual".to_string(),
    }
}

fn build_uc(
    users: UserRepoStub, sessions: SessionRepoStub, refresh: RefreshRepoStub, hasher: PasswordHasherStub,
    refresh_factory: RefreshTokenFactoryStub, cache: RevocationCacheStub, email_verification: EmailVerificationRepoStub,
    mailer: MailerStub, tax_profiles: TaxProfileClientStub,
) -> TestAuthUseCases {
    AuthUseCases {
        users,
        sessions,
        refresh,
        hasher,
        access: AccessTokenIssuerStub,
        refresh_factory,
        cache,
        email_verification,
        mailer,
        tax_profiles,
        verify_config: VerifyEmailConfig {
            base_url: "http://localhost:8085/auth/verify?token=".to_string(),
            token_ttl_secs: 60 * 60,
        },
        access_ttl: 900,
        refresh_ttl: 2_592_000,
        dummy_password_hash: "$argon2id$v=19$m=65536,t=3,p=1$R0VORVJBVEVEX1NBTFQ$8v0QWnN8S2sRzR2VdX1lA4O3p2y1W8Q4G8g7w8r2s1U"
            .to_string(),
    }
}

#[tokio::test]
async fn register_new_user_success_flow() {
    let user_id = Uuid::new_v4();
    let users = UserRepoStub {
        create_user_res: Mutex::new(Some(Ok(Some(user_id)))),
        ..Default::default()
    };
    let tax_profiles = TaxProfileClientStub {
        upsert_res: Mutex::new(Some(Ok(()))),
        ..Default::default()
    };
    let email_verification = EmailVerificationRepoStub {
        create_token_res: Mutex::new(Some(Ok(()))),
        ..Default::default()
    };
    let mailer = MailerStub {
        send_verification_res: Mutex::new(Some(Ok(()))),
        ..Default::default()
    };
    let hasher = PasswordHasherStub {
        hash_res: Mutex::new(Some(Ok("hashed-password".to_string()))),
        ..Default::default()
    };
    let refresh_factory = RefreshTokenFactoryStub {
        hash_res: Mutex::new(Some(vec![1, 2, 3, 4])),
    };

    let upsert_calls = tax_profiles.upsert_calls.clone();
    let create_token_calls = email_verification.create_token_calls.clone();
    let send_calls = mailer.send_verification_calls.clone();
    let last_upsert_user_id = tax_profiles.last_upsert_user_id.lock().expect("mutex poisoned").take();
    assert!(last_upsert_user_id.is_none());

    let uc = build_uc(
        users,
        SessionRepoStub::default(),
        RefreshRepoStub::default(),
        hasher,
        refresh_factory,
        RevocationCacheStub::default(),
        email_verification,
        mailer,
        tax_profiles,
    );

    let res = uc.register("  USER@example.com  ", "W7!fPq2#Kb9@Lm4$Tx", &dummy_tax_profile()).await;
    assert!(res.is_ok(), "register should succeed");

    assert_eq!(upsert_calls.load(Ordering::Relaxed), 1);
    assert_eq!(create_token_calls.load(Ordering::Relaxed), 1);
    assert_eq!(send_calls.load(Ordering::Relaxed), 1);

    let user_id_in_tax_profile = uc.tax_profiles.last_upsert_user_id.lock().expect("mutex poisoned").to_owned();
    assert_eq!(user_id_in_tax_profile, Some(user_id));
}

#[tokio::test]
async fn register_rolls_back_pending_user_when_tax_profile_sync_fails() {
    let user_id = Uuid::new_v4();
    let users = UserRepoStub {
        create_user_res: Mutex::new(Some(Ok(Some(user_id)))),
        delete_pending_user_res: Mutex::new(Some(Ok(true))),
        ..Default::default()
    };
    let delete_calls = users.delete_pending_user_calls.clone();

    let tax_profiles = TaxProfileClientStub {
        upsert_res: Mutex::new(Some(Err(AuthError::Storage("tax profile sync failed".to_string())))),
        ..Default::default()
    };

    let hasher = PasswordHasherStub {
        hash_res: Mutex::new(Some(Ok("hashed-password".to_string()))),
        ..Default::default()
    };
    let refresh_factory = RefreshTokenFactoryStub::default();

    let uc = build_uc(
        users,
        SessionRepoStub::default(),
        RefreshRepoStub::default(),
        hasher,
        refresh_factory,
        RevocationCacheStub::default(),
        EmailVerificationRepoStub::default(),
        MailerStub::default(),
        tax_profiles,
    );

    let err =
        uc.register("user@example.com", "W7!fPq2#Kb9@Lm4$Tx", &dummy_tax_profile()).await.expect_err("register should fail");

    assert!(matches!(err, AuthError::Storage(_)));
    assert_eq!(delete_calls.load(Ordering::Relaxed), 1, "pending user must be removed on sync failure");
}

#[tokio::test]
async fn login_rejects_invalid_password() {
    let user = UserWithHash {
        id: Uuid::new_v4(),
        email: "user@example.com".to_string(),
        status: UserStatus::Active,
        created_at: Utc::now(),
        password_hash: "stored-hash".to_string(),
    };

    let users = UserRepoStub {
        find_by_email_res: Mutex::new(Some(Ok(Some(user)))),
        ..Default::default()
    };
    let hasher = PasswordHasherStub {
        verify_res: Mutex::new(Some(Ok(false))),
        ..Default::default()
    };

    let uc = build_uc(
        users,
        SessionRepoStub::default(),
        RefreshRepoStub::default(),
        hasher,
        RefreshTokenFactoryStub::default(),
        RevocationCacheStub::default(),
        EmailVerificationRepoStub::default(),
        MailerStub::default(),
        TaxProfileClientStub::default(),
    );

    let err = uc.login("user@example.com", "wrong-password", None, None).await.expect_err("login should fail");
    assert!(matches!(err, AuthError::InvalidCredentials));
}

#[tokio::test]
async fn refresh_rejects_oversized_token() {
    let uc = build_uc(
        UserRepoStub::default(),
        SessionRepoStub::default(),
        RefreshRepoStub::default(),
        PasswordHasherStub::default(),
        RefreshTokenFactoryStub::default(),
        RevocationCacheStub::default(),
        EmailVerificationRepoStub::default(),
        MailerStub::default(),
        TaxProfileClientStub::default(),
    );

    let too_long = "a".repeat(2049);
    let err = uc.refresh(&too_long).await.expect_err("oversized token must be rejected");
    assert!(matches!(err, AuthError::TokenInvalid));
}

#[tokio::test]
async fn refresh_detects_reuse_from_cache_and_revokes_session() {
    let refresh_token = RefreshToken {
        jti: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        expires_at: Utc::now() + Duration::hours(1),
        rotated_at: None,
        revoked_at: None,
    };

    let refresh_repo = RefreshRepoStub {
        get_by_hash_res: Mutex::new(Some(Ok(Some(refresh_token.clone())))),
        revoke_all_for_session_res: Mutex::new(Some(Ok(()))),
        ..Default::default()
    };
    let refresh_revoke_calls = refresh_repo.revoke_all_for_session_calls.clone();

    let session_repo = SessionRepoStub {
        set_status_res: Mutex::new(Some(Ok(()))),
        ..Default::default()
    };
    let set_status_calls = session_repo.set_status_calls.clone();

    let cache = RevocationCacheStub {
        check_refresh_res: Mutex::new(Some(Ok(Some(RefreshBlockReason::Rotated)))),
        revoke_all_for_session_res: Mutex::new(Some(Ok(()))),
        ..Default::default()
    };
    let cache_revoke_calls = cache.revoke_all_for_session_calls.clone();

    let refresh_factory = RefreshTokenFactoryStub {
        hash_res: Mutex::new(Some(vec![9, 9, 9, 9])),
    };

    let uc = build_uc(
        UserRepoStub::default(),
        session_repo,
        refresh_repo,
        PasswordHasherStub::default(),
        refresh_factory,
        cache,
        EmailVerificationRepoStub::default(),
        MailerStub::default(),
        TaxProfileClientStub::default(),
    );

    let err = uc.refresh("refresh-token").await.expect_err("reused token must be rejected");
    assert!(matches!(err, AuthError::TokenReuse));
    assert_eq!(refresh_revoke_calls.load(Ordering::Relaxed), 1);
    assert_eq!(set_status_calls.load(Ordering::Relaxed), 1);
    assert_eq!(cache_revoke_calls.load(Ordering::Relaxed), 1);
}
