pub mod color;
pub mod command;
pub mod event_handle;

//use tracing::debug;

use crate::{
    errors::{ColcatError, RendererError},
    ratatui::command::Commands,
    renderer::{KeyComboMode, RatatuiApp, SearchModePhase},
};

pub trait RatatuiAppFunctionality {
    fn search_text(&mut self) -> Result<(), ColcatError>;
    fn handle_command(&mut self, command: Commands) -> Result<(), ColcatError>;
}

impl RatatuiAppFunctionality for RatatuiApp {
    fn search_text(&mut self) -> Result<(), ColcatError> {
        let KeyComboMode::Search(SearchModePhase::Typing(text)) =
            &self.state.display_state.key_state
        else {
            return Err(RendererError::TriedSearchWhileSearching.into());
        };
        let hits = self.state.text_state.current_text.match_indices(text);
        Err(ColcatError::Unimplemented(
            "RatatuiAppFunctionality::search_text".to_string(),
        ))
        //debug!("search_text text: {text:?}, hits in search_text: {hits:#?}");
        //TODO
    }
    fn handle_command(&mut self, command: Commands) -> Result<(), ColcatError> {
        match command {
            Commands::Help => self.show_notification("Help".to_string(), Commands::help_text()),
            #[cfg(feature = "semantics")]
            Commands::Semantics(new_string) => {
                self.state
                    .text_state
                    .set_semantics_target_string(new_string)?;
                self.state.process_text()
            }
        }
    }
}
