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
