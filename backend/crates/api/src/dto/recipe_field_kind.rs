use orchestrata_recipes::FieldKind;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RecipeFieldKind {
    String,
    Integer,
    Boolean,
    StringList,
    IntegerList,
}

impl From<FieldKind> for RecipeFieldKind {
    fn from(kind: FieldKind) -> Self {
        match kind {
            FieldKind::String => Self::String,
            FieldKind::Integer => Self::Integer,
            FieldKind::Boolean => Self::Boolean,
            FieldKind::StringList => Self::StringList,
            FieldKind::IntegerList => Self::IntegerList,
        }
    }
}
