use std::{error::Error, fs, path::PathBuf, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use uuid::Uuid;

use crate::docker_fixture::Docker;

pub struct PostgresFixture {
    docker: Docker,
    id: String,
    password: String,
    port: u16,
}

impl PostgresFixture {
    pub async fn start() -> Result<Self, Box<dyn Error>> {
        let docker = Docker::local()?;
        if docker
            .run(
                &[
                    "image",
                    "inspect",
                    "postgres:17-alpine",
                    "--format",
                    "{{.Id}}",
                ],
                Duration::from_secs(10),
            )
            .is_err()
        {
            docker.run(
                &["pull", "--quiet", "postgres:17-alpine"],
                Duration::from_secs(180),
            )?;
        }
        let password = Uuid::new_v4().simple().to_string();
        let environment = EnvironmentFile::new(&password)?;
        let mut fixture = Self {
            docker,
            id: format!("orchestrata-session-test-{}", Uuid::now_v7().simple()),
            password,
            port: 0,
        };
        let name = fixture.id.clone();
        let id = fixture.docker.run(
            &[
                "create",
                "--rm",
                "--pull",
                "never",
                "--name",
                &name,
                "--publish",
                "127.0.0.1::5432",
                "--label",
                "orchestrata.test=session-store",
                "--env-file",
                environment
                    .0
                    .to_str()
                    .ok_or("invalid environment file path")?,
                "postgres:17-alpine",
            ],
            Duration::from_secs(30),
        )?;
        let id = id.trim();
        if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Docker returned an invalid container identifier".into());
        }
        fixture.id = id.to_owned();
        fixture
            .docker
            .run(&["start", &fixture.id], Duration::from_secs(30))?;
        let address = fixture
            .docker
            .run(&["port", &fixture.id, "5432/tcp"], Duration::from_secs(10))?;
        fixture.port = address
            .trim()
            .strip_prefix("127.0.0.1:")
            .ok_or("PostgreSQL must bind only to loopback")?
            .parse()?;
        if fixture.port == 0 {
            return Err("Docker did not assign a PostgreSQL port".into());
        }
        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                if let Ok(connection) = fixture.connect().await
                    && connection.ping().await.is_ok()
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        })
        .await
        .map_err(|_| "disposable PostgreSQL did not become ready")?;
        Ok(fixture)
    }

    pub fn remove(&self) -> Result<(), Box<dyn Error>> {
        self.docker
            .run(&["rm", "--force", &self.id], Duration::from_secs(10))?;
        Ok(())
    }

    pub async fn connect(&self) -> Result<DatabaseConnection, Box<dyn Error>> {
        let mut options = ConnectOptions::new(format!(
            "postgres://orchestrata:{}@127.0.0.1:{}/orchestrata_test?sslmode=disable",
            self.password, self.port,
        ));
        options
            .connect_timeout(Duration::from_secs(2))
            .max_connections(2)
            .min_connections(0)
            .sqlx_logging(false);
        Database::connect(options)
            .await
            .map_err(|_| "test database connection failed".into())
    }
}

impl Drop for PostgresFixture {
    fn drop(&mut self) {
        let _ = self
            .docker
            .run(&["rm", "--force", &self.id], Duration::from_secs(10));
    }
}

struct EnvironmentFile(PathBuf);

impl EnvironmentFile {
    fn new(password: &str) -> Result<Self, Box<dyn Error>> {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!("orchestrata-pg-{}", Uuid::now_v7()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut output = options.open(&path)?;
        let file = Self(path);
        write!(
            output,
            "POSTGRES_DB=orchestrata_test\nPOSTGRES_USER=orchestrata\nPOSTGRES_PASSWORD={password}\n"
        )?;
        Ok(file)
    }
}

impl Drop for EnvironmentFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
