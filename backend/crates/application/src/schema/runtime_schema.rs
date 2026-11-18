use std::collections::HashMap;

use super::{ObjectDefinition, ObjectId, SchemaError, validation};

pub struct RuntimeSchema {
    definitions: HashMap<ObjectId, ObjectDefinition>,
}

impl RuntimeSchema {
    /// Validates a complete in-memory schema, including references among declarations.
    /// This does not persist definitions, validate records, or activate triggers.
    pub fn compile(definitions: Vec<ObjectDefinition>) -> Result<Self, SchemaError> {
        validation::validate(&definitions)?;

        Ok(Self {
            definitions: definitions
                .into_iter()
                .map(|definition| (definition.id, definition))
                .collect(),
        })
    }

    pub fn definition(&self, id: ObjectId) -> Option<&ObjectDefinition> {
        self.definitions.get(&id)
    }
}
