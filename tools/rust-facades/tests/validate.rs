use orchestrata_facade_check::violations;

#[test]
fn accepts_wiring_documentation_and_configuration() {
    let source = r#"
        //! A function name in documentation: fn example() {}
        #![deny(unsafe_code)]
        #[path = "implementation.rs"] mod implementation;
        #[cfg_attr(test, allow(unused_imports))]
        pub use implementation::{run, Model};
        extern crate core;
    "#;
    assert!(violations(source).unwrap().is_empty());
}

#[test]
fn rejects_functions_impls_and_every_substantive_item_kind() {
    for source in [
        "pub async unsafe fn run() {}",
        "impl Model { fn run(&self) {} }",
        "pub struct Model;",
        "pub enum Model { A }",
        "union Model { x: u32 }",
        "trait Model {}",
        "type Model = u32;",
        "const LIMIT: usize = 1;",
        "static LIMIT: usize = 1;",
        "unsafe extern \"C\" { fn run(); }",
    ] {
        assert!(!violations(source).unwrap().is_empty(), "{source}");
    }
}

#[test]
fn rejects_inline_modules_macros_and_disabled_implementation() {
    for source in [
        "mod nested { pub fn run() {} }",
        "mod nested { pub use core::fmt; }",
        "include!(\"implementation.rs\");",
        "macro_rules! generate { () => { fn run() {} } }",
        "#[cfg(any())] fn disabled() {}",
        "#[cfg(test)] mod tests { #[test] fn run() {} }",
        "const _: () = { fn hidden() {} };",
    ] {
        assert!(!violations(source).unwrap().is_empty(), "{source}");
    }
}

#[test]
fn rejects_code_generating_attributes_and_parse_failures() {
    for source in [
        "#[generate] mod external;",
        "#[cfg_attr(test, generate)] mod external;",
        "#[cfg_attr(test, cfg_attr(any(), generate))] mod external;",
    ] {
        assert!(!violations(source).unwrap().is_empty(), "{source}");
    }
    assert!(violations("pub fn broken(").is_err());
}
