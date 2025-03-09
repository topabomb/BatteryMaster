
pub trait Status<T> {
    fn build() -> Option<T>;
}
pub trait Last{
    fn last(&mut self);
}