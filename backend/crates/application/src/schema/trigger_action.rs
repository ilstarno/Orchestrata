#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TriggerAction {
    /// Declares a logical event only. Compilation does not execute or publish it.
    EmitEvent { key: String },
}
