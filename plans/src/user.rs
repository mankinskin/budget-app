#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: usize,
    name: String,
}

impl From<String> for User {
    fn from(s: String) -> Self {
        User::new(s)
    }
}
impl From<&str> for User {
    fn from(s: &str) -> Self {
        User::new(s)
    }
}
use std::fmt::{self, Display};
impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl User {
    pub fn new<S: ToString>(name: S) -> Self {
        Self {
            name: name.to_string(),
            id: 0,
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}
