use crate::{
    errors::ColcatError,
    types::{TextInput, state::ColcatMode},
};

pub trait StateProcess {
    fn initial_process(&self, input: TextInput) -> Result<String, ColcatError>;
}

impl StateProcess for ColcatMode {
    fn initial_process(&self, input: TextInput) -> Result<String, ColcatError> {
        Ok(input.text().clone())
    }
}
