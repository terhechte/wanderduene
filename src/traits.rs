pub trait DunePathBuilder {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self;
}
