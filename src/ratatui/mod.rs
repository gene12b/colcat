pub mod color;
pub mod event_handle;

//use tracing::debug;

use crate::{
    errors::{ColcatError, RendererError},
    renderer::{KeyComboMode, RatatuiApp, SearchModePhase},
};

pub trait RatatuiAppFunctionality {
    fn search_text(&mut self) -> Result<(), ColcatError>;
}

impl RatatuiAppFunctionality for RatatuiApp {
    fn search_text(&mut self) -> Result<(), ColcatError> {
        let KeyComboMode::Search(SearchModePhase::Typing(text)) =
            &self.state.display_state.key_state
        else {
            return Err(RendererError::TriedSearchWhileSearching.into());
        };
        let hits = self.state.text_state.current_text.match_indices(text);
        //debug!("search_text text: {text:?}, hits in search_text: {hits:#?}");
        //TODO
        Ok(())
    }
}
