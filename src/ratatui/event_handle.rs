use std::str::FromStr;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    errors::ColcatError,
    ratatui::{RatatuiAppFunctionality, command::Commands},
    renderer::{CommandModePhase, KeyComboMode, RatatuiApp, SearchModePhase},
};

pub trait RatatuiKeyEventHandle {
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<(), ColcatError>;
}

impl RatatuiKeyEventHandle for RatatuiApp {
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<(), ColcatError> {
        //check if need to break from modes into KeyComboMode::View
        if key_event.code == KeyCode::Esc {
            self.state.display_state.escape_pressed();
            Ok(())
        } else {
            match self.state.display_state.key_state {
                KeyComboMode::Go => handle_go(self, key_event),
                KeyComboMode::Search(_) => handle_search(self, key_event),
                KeyComboMode::Command(_) => handle_command(self, key_event),
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
                KeyCode::Backspace => {
                    text.pop();
                }
                KeyCode::Char(other) => text.push(other),
                _ => {}
            },
            SearchModePhase::Searching {
                hits: _,
                hit_idx: _,
            } => {
                //TODO: jump between search hits
                return Err(ColcatError::Unimplemented(
                    "handle_search SearchModePhase::Searching".to_string(),
                ));
            }
        },
        _ => {
            //Err
        }
    }
    Ok(())
}
fn handle_command(app: &mut RatatuiApp, key_event: KeyEvent) -> Result<(), ColcatError> {
    if let KeyComboMode::Command(phase) = &mut app.state.display_state.key_state {
        match phase {
            CommandModePhase::Typing(val) => match key_event.code {
                KeyCode::Enter => {
                    let command = Commands::from_str(val)?;
                    app.handle_command(command)?;
                    app.state.display_state.escape_pressed();
                }
                KeyCode::Backspace => {
                    val.pop();
                }
                KeyCode::Char(other) => {
                    val.push(other);
                }
                _ => {}
            },
        }
    }
    Ok(())
}

fn handle_default(app: &mut RatatuiApp, key_event: KeyEvent) -> Result<(), ColcatError> {
    match key_event.code {
        KeyCode::Char('q') => app.exit(),
        KeyCode::Char('j') => {
            app.state.display_state.position.line_number =
                (app.state.display_state.position.line_number + 1).min(app.get_last_line() + 15)
        }
        KeyCode::Char('k') => {
            app.state.display_state.position.line_number =
                (app.state.display_state.position.line_number - 1).max(0)
        }
        KeyCode::Char('u') => {
            app.state.display_state.position.line_number =
                (app.state.display_state.position.line_number - 10).max(0)
        }
        KeyCode::Char('d') => {
            app.state.display_state.position.line_number =
                (app.state.display_state.position.line_number + 10).min(app.get_last_line() + 15)
        }
        KeyCode::Char('G') => app.state.display_state.position.line_number = app.get_last_line(),
        KeyCode::Char(other) => app.state.display_state.key_state = other.into(),
        _ => {}
    }
    Ok(())
}
