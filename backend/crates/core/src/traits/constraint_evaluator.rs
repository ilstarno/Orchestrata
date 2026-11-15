use crate::score::HardMediumSoftScore;

pub trait ConstraintEvaluator<P> {
    type Error;

    fn evaluate(&self, problem: &P) -> Result<HardMediumSoftScore, Self::Error>;
}
