pub trait PlanningEntity {
    type Id: Copy + Eq;

    fn id(&self) -> Self::Id;
}
