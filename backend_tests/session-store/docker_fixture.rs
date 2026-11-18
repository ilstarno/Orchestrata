use std::{
    error::Error,
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub struct Docker {
    endpoint: String,
}

impl Docker {
    pub fn local() -> Result<Self, Box<dyn Error>> {
        if !cfg!(unix) {
            return Err("the PostgreSQL fixture requires a local Unix Docker socket".into());
        }
        let host = std::env::var("DOCKER_HOST").ok().filter(|s| !s.is_empty());
        if let Some(host) = &host {
            validate_endpoint(host)?;
        }
        let context = bounded(
            Command::new("docker").args([
                "context",
                "inspect",
                "--format",
                "{{.Endpoints.docker.Host}}",
            ]),
            Duration::from_secs(10),
        )?;
        validate_endpoint(context.trim())?;
        let explicit_context = std::env::var("DOCKER_CONTEXT").is_ok_and(|s| !s.is_empty());
        let endpoint = if explicit_context {
            context.trim().to_owned()
        } else {
            host.unwrap_or_else(|| context.trim().to_owned())
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let path = validate_endpoint(&endpoint)?;
            if !std::fs::metadata(path)?.file_type().is_socket() {
                return Err("Docker endpoint is not a local socket".into());
            }
        }
        Ok(Self { endpoint })
    }

    pub fn run(&self, arguments: &[&str], timeout: Duration) -> Result<String, Box<dyn Error>> {
        bounded(
            Command::new("docker")
                .env_remove("DOCKER_HOST")
                .env_remove("DOCKER_CONTEXT")
                .env_remove("DOCKER_TLS_VERIFY")
                .env_remove("DOCKER_CERT_PATH")
                .args(["--host", &self.endpoint])
                .args(arguments),
            timeout,
        )
    }
}

fn validate_endpoint(endpoint: &str) -> Result<&str, Box<dyn Error>> {
    let path = endpoint
        .strip_prefix("unix://")
        .filter(|p| p.starts_with('/') && !p.contains('\0'))
        .ok_or("remote Docker endpoints are not allowed for this fixture")?;
    Ok(path)
}

fn bounded(command: &mut Command, timeout: Duration) -> Result<String, Box<dyn Error>> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "could not start fixture command")?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err("fixture command exceeded its deadline".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    if !status.success() {
        return Err("fixture command failed".into());
    }
    let mut output = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing fixture output")?
        .take(8193)
        .read_to_end(&mut output)?;
    if output.len() > 8192 {
        return Err("fixture output exceeded its limit".into());
    }
    Ok(String::from_utf8(output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_endpoints_are_rejected_before_container_commands() {
        for endpoint in [
            "tcp://127.0.0.1:2375",
            "ssh://example.invalid",
            "https://example.invalid",
            "npipe://pipe/docker",
            "unix://relative",
        ] {
            assert!(validate_endpoint(endpoint).is_err());
        }
        assert_eq!(
            validate_endpoint("unix:///var/run/docker.sock").unwrap(),
            "/var/run/docker.sock"
        );
    }

    #[cfg(unix)]
    #[test]
    fn stalled_command_is_killed_within_a_bounded_time() {
        let start = Instant::now();
        assert!(bounded(Command::new("sleep").arg("10"), Duration::from_millis(50)).is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
