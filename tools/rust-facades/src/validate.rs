use syn::{Attribute, Item, Meta, Token, punctuated::Punctuated};

/// Parse every item, including conditionally compiled items, without expanding macros.
pub fn violations(source: &str) -> Result<Vec<String>, syn::Error> {
    let file = syn::parse_file(source)?;
    let mut violations = Vec::new();
    check_attributes(&file.attrs, &mut violations);
    for item in file.items {
        let attributes = match item {
            Item::Use(item) => item.attrs,
            Item::ExternCrate(item) => item.attrs,
            Item::Mod(item) if item.content.is_none() => item.attrs,
            item => {
                let kind = match item {
                    Item::Fn(_) => "function",
                    Item::Impl(_) => "impl",
                    Item::Struct(_) => "struct",
                    Item::Enum(_) => "enum",
                    Item::Union(_) => "union",
                    Item::Trait(_) => "trait",
                    Item::TraitAlias(_) => "trait alias",
                    Item::Type(_) => "type alias",
                    Item::Const(_) => "constant",
                    Item::Static(_) => "static",
                    Item::Macro(_) => "macro",
                    Item::Mod(_) => "inline module",
                    Item::ForeignMod(_) => "foreign block",
                    _ => "unsupported item",
                };
                violations.push(format!("{kind} must be in a dedicated implementation file"));
                continue;
            }
        };
        check_attributes(&attributes, &mut violations);
    }
    Ok(violations)
}

fn check_attributes(attributes: &[Attribute], violations: &mut Vec<String>) {
    for attribute in attributes {
        if !configuration_attribute(&attribute.meta) {
            violations.push("unsupported attribute may generate implementation".into());
        }
    }
}

fn configuration_attribute(meta: &Meta) -> bool {
    let path = meta.path();
    if path.is_ident("cfg_attr") {
        let Meta::List(list) = meta else {
            return false;
        };
        let Ok(arguments) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            return false;
        };
        return arguments.len() >= 2 && arguments.iter().skip(1).all(configuration_attribute);
    }
    [
        "doc",
        "path",
        "cfg",
        "allow",
        "warn",
        "deny",
        "forbid",
        "expect",
        "no_std",
        "no_implicit_prelude",
        "recursion_limit",
        "type_length_limit",
        "macro_use",
    ]
    .iter()
    .any(|name| path.is_ident(name))
}
