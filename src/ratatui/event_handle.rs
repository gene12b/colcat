use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    errors::ColcatError,
    ratatui::RatatuiAppFunctionality,
    renderer::{KeyComboMode, RatatuiApp, SearchModePhase},
};

pub trait RatatuiKeyEventHandle {
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<(), ColcatError>;
}

impl RatatuiKeyEventHandle for RatatuiApp {
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<(), ColcatError> {
        //check if need to break from modes into KeyComboMode::View
        if key_event.code == KeyCode::Esc {
            self.state.display_state.key_state = KeyComboMode::View;
            Ok(())
        } else {
            match self.state.display_state.key_state {
                KeyComboMode::Go => handle_go(self, key_event),
                KeyComboMode::Search(_) => handle_search(self, key_event),
                _ => handle_default(self, key_event),
            }
        }
    }
}

fn handle_go(app: &mut RatatuiApp, key_event: KeyEvent) -> Result<(), ColcatError> {
    match key_event.code {
        KeyCode::Char('g') => {
            app.state.display_state.position.line_number = 0;
        }
        _ => handle_default(app, key_event)?,
    }
    app.state.display_state.key_state = KeyComboMode::View;
    Ok(())
}
fn handle_search(app: &mut RatatuiApp, key_event: KeyEvent) -> Result<(), ColcatError> {
    match &mut app.state.display_state.key_state {
        KeyComboMode::Search(phase) => match phase {
            SearchModePhase::Typing(text) => match key_event.code {
                KeyCode::Enter => app.search_text()?,
                KeyCode::Char(other) => text.push(other),
                _ => {}
            },
            SearchModePhase::Searching { hits, hit_idx } => {
                //TODO: jump between search hits
            }
        },
        _ => {
            //Err
        }
    }
    Ok(())
}

fn handle_default(app: &mut RatatuiApp, key_event: KeyEvent) -> Result<(), ColcatError> {
    match key_event.code {
        KeyCode::Char('q') => app.exit(),
        KeyCode::Char('j') => app.state.display_state.position.line_number += 1,
        KeyCode::Char('k') => app.state.display_state.position.line_number -= 1,
        KeyCode::Char('u') => app.state.display_state.position.line_number -= 10,
        KeyCode::Char('d') => app.state.display_state.position.line_number += 10,
        KeyCode::Char('G') => app.state.display_state.position.line_number = app.get_last_line(),
        KeyCode::Char(other) => app.state.display_state.key_state = other.into(),
        _ => {}
    }
    Ok(())
}
