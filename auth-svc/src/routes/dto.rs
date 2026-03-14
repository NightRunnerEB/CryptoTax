use serde::Deserialize;

use crate::auth_core::models::RegisterTaxProfile;

/// ----- DTOs -----
#[derive(Deserialize)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
    pub tax_profile: RegisterTaxProfile,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
    pub ip: Option<String>,
    pub ua: Option<String>,
}

#[derive(Deserialize)]
pub struct RefreshReq {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailReq {
    pub token: String,
}
