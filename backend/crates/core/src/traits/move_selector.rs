pub trait MoveSelector<S> {
    type Move;

    fn next_move(&mut self, state: &S) -> Option<Self::Move>;
}
