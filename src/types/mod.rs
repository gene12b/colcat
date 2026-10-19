pub mod comm;
pub mod indexing;
pub mod iter;
pub mod params;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod state;

#[derive(Debug, Clone)]
pub struct TextInput {
    text: String,
}

impl TextInput {
    pub fn text(&self) -> &String {
        &self.text
    }
}

impl From<String> for TextInput {
    fn from(value: String) -> Self {
        Self { text: value }
    }
}

impl TextInput {
    pub fn new(text: String) -> Self {
        Self::from(text)
    }
}

pub trait TryIntoExt {
    type Error;
    type Output;
    fn try_into_ext(self) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;
}
///Into extension trait in order to define Into for external structs/enums
pub trait IntoExt {
    type Output;
    fn into_ext(self) -> Self::Output
    where
        Self: Sized;
}
