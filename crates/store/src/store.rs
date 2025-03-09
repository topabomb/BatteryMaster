pub struct Store {}
impl Default for Store {
    fn default() -> Self {
        Self {}
    }
}
impl Store {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self) {}
}
