pub(super) struct BuiltinRecipe {
    pub id: &'static str,
    pub xml: &'static str,
    pub problem: &'static str,
    #[cfg(test)]
    pub expected: &'static str,
}
