use std::time::Duration;

pub(super) const MAX_CONFIGURED_URL_BYTES: usize = 2_048;
pub(super) const MAX_CLIENT_ID_BYTES: usize = 512;
pub(super) const MAX_AUTHORIZATION_CODE_BYTES: usize = 4_096;
pub(super) const SIGN_IN_LIFETIME: Duration = Duration::from_secs(300);

pub(super) const MAX_TOKEN_RESPONSE_BYTES: usize = 64 * 1024;
pub(super) const MAX_VERIFICATION_KEYS: usize = 32;
pub(super) const MAX_SUBJECT_BYTES: usize = 255;
