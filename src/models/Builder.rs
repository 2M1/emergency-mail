pub trait Builder<T> {
    type E;

    fn build(&self) -> Result<T, Self::E>;
}
