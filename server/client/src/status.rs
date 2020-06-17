
#[derive(Clone)]
pub enum Status<T>
    where T: Clone,
{
    Empty,
    Loading,
    Ready(T),
    Failed(String),
}

impl<T> Default for Status<T>
    where T: Clone,
{
    fn default() -> Self {
        Self::Empty
    }
}