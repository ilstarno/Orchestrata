/// Requested resource scope. This value is not evidence of permission.
#[derive(Debug, Clone, Copy)]
pub struct RunScope<'a> {
    pub workspace: &'a str,
    pub package: &'a str,
    pub entrypoint: &'a str,
}
