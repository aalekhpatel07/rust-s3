

#[derive(Debug, PartialEq, Eq)]
pub struct Tag {
    pub(crate) key: String,
    pub(crate) value: String,
}

impl Tag {
    pub fn key(&self) -> String {
        self.key.to_owned()
    }

    pub fn value(&self) -> String {
        self.value.to_owned()
    }
}