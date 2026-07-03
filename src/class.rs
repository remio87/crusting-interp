#[derive(Debug)]
pub struct Class {
    pub name: String,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Class {
    pub fn new(name: &str) -> Self {
        Class {
            name: name.to_string(),
        }
    }
}
