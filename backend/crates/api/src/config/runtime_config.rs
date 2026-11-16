use std::{
    env,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use super::RuntimeConfigError;

pub(super) const HTTP_BIND_ENV: &str = "ORCHESTRATA_HTTP_BIND";
const DEFAULT_HTTP_BIND: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000));

pub struct RuntimeConfig {
    pub http_bind: SocketAddr,
}

impl RuntimeConfig {
    pub fn from_environment() -> Result<Self, RuntimeConfigError> {
        Self::from_env_value(env::var(HTTP_BIND_ENV))
    }

    fn from_env_value(value: Result<String, env::VarError>) -> Result<Self, RuntimeConfigError> {
        let http_bind = match value {
            Ok(value) => value
                .parse()
                .map_err(|_| RuntimeConfigError::InvalidHttpBind)?,
            Err(env::VarError::NotPresent) => DEFAULT_HTTP_BIND,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(RuntimeConfigError::NonUnicodeHttpBind);
            }
        };
        Ok(Self { http_bind })
    }
}

#[cfg(test)]
#[path = "../../../../../backend_tests/api/runtime_config_unit.rs"]
mod tests;
