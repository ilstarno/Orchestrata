env "local" {
  url = getenv("DATABASE_URL")
  dev = "docker://postgres/17/dev?search_path=public"

  schema {
    src = "file://schema"
  }

  migration {
    dir = "file://migrations"
  }
}
