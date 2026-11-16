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
mod tests {
    use super::*;

    #[test]
    fn defaults_to_local_development_address() {
        let config = RuntimeConfig::from_env_value(Err(env::VarError::NotPresent)).unwrap();
        assert_eq!(config.http_bind.to_string(), "127.0.0.1:3000");
    }

    #[test]
    fn accepts_explicit_ipv4_and_ipv6_socket_addresses() {
        for address in ["0.0.0.0:8080", "[::]:8443"] {
            let config = RuntimeConfig::from_env_value(Ok(address.to_owned())).unwrap();
            assert_eq!(config.http_bind.to_string(), address);
        }
    }

    #[test]
    fn rejects_malformed_addresses_instead_of_silently_using_the_default() {
        for address in ["", "8080", "::1", "api:8080", "0.0.0.0:65536"] {
            assert!(matches!(
                RuntimeConfig::from_env_value(Ok(address.to_owned())),
                Err(RuntimeConfigError::InvalidHttpBind)
            ));
        }
    }

    #[test]
    fn explains_non_unicode_environment_values() {
        assert!(matches!(
            RuntimeConfig::from_env_value(Err(env::VarError::NotUnicode("invalid".into()))),
            Err(RuntimeConfigError::NonUnicodeHttpBind)
        ));
    }
}
