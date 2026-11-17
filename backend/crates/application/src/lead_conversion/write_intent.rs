/// An explicit choice within a synthetic replay. Not a production authorization grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteIntent {
    PreviewOnly,
    ApplyToFixture,
}
