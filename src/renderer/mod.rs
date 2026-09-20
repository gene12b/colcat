#[cfg(feature = "pdf")]
pub mod pdf;

use std::str::FromStr;

use crossterm::event::{self, Event, KeyEventKind};
use itertools::Itertools;
use ratatui::{
    DefaultTerminal, Frame,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Gauge, Paragraph, Widget, Wrap},
};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator, VariantNames};
use tracing::debug;

#[cfg(feature = "semantics")]
use crate::color::semantics::ProcessedSemantics;
use crate::{
    color::{ColoredCharacter, ColoringScheme},
    errors::{ColcatError, UnwrapFallible},
    ratatui::event_handle::RatatuiKeyEventHandle,
    types::{
        IntoExt,
        params::ColcatParams,
        state::{ColcatMode, ColcatState, TextState},
    },
};

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, AsRefStr, VariantNames))]
#[strum(serialize_all = "snake_case")]
pub enum Renderer {
    Ratatui(Box<RatatuiApp>),
    #[cfg(feature = "pdf")]
    Pdf,
}

impl FromStr for RendererDiscriminants {
    type Err = ColcatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(var) = RendererDiscriminants::iter().find(|val| s == val.as_ref()) {
            Ok(var)
        } else {
            Err(ColcatError::FromStrParse(
                "RendererDiscriminants".to_string(),
                s.to_string(),
                RendererDiscriminants::VARIANTS
                    .iter()
                    .map(|val| val.to_string())
                    .collect_vec(),
            ))
        }
    }
}

impl Renderer {
    pub fn new(text: String, params: ColcatParams) -> Result<Self, ColcatError> {
        Ok(Self::Ratatui(Box::new(RatatuiApp::new(text, params)?)))
    }
}

impl Renderer {
    pub fn run(self) -> Result<(), color_eyre::eyre::Report> {
        match self {
            Self::Ratatui(mut app) => {
                color_eyre::install()?;
                let app_res = ratatui::run(|term| app.run(term));
                ratatui::restore();
                app_res
            }
            #[cfg(feature = "pdf")]
            Self::Pdf => {
                todo!("impl Renderer::Pdf::run()");
            }
        }
    }
}

#[derive(Debug)]
pub struct RatatuiApp {
    pub state: ColcatState,
}

#[derive(Debug, Default)]
pub enum KeyComboMode {
    #[default]
    View,
    //Insert,
    Go,
    Search(SearchModePhase),
}

#[derive(Debug)]
pub enum SearchModePhase {
    Typing(String),
    Searching { hits: Vec<usize>, hit_idx: usize },
}

impl Default for SearchModePhase {
    fn default() -> Self {
        Self::Typing(String::new())
    }
}

impl From<char> for KeyComboMode {
    fn from(value: char) -> Self {
        match value {
            'g' => Self::Go,
            '/' => Self::Search(Default::default()),
            _ => Self::View,
        }
    }
}

impl RatatuiApp {
    pub fn new(text: String, params: ColcatParams) -> Result<Self, ColcatError> {
        Ok(Self {
            state: ColcatState::new(params, text)?,
        })
    }
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), color_eyre::eyre::Report> {
        debug!("state: {:#?}", self.state);
        //debug!(
        //    "params json:\n{}",
        //    serde_json::to_string_pretty(&self.params).unwrap()
        //);
        //debug!("current_text: {:#?}", self.current_text);
        //debug!("processed_text(before): {:#?}", self.processed_text);
        while !self.state.should_exit() {
            terminal.draw(|frame| self.draw(frame))?;
            if self.state.needs_processing() {
                self.state.process_text().map_err_fallible()?;
                self.state.text_state.needs_processing = false;
            }
            //debug!("processed_text(after): {:#?}", self.processed_text);
            self.handle_events().map_err_fallible()?;
        }
        Ok(())
    }

    //fn process_text(&mut self, text: String) -> Result<(), ColcatError> {
    //    //let current_text = &self.current_text;
    //    let params = &self.params;
    //    #[cfg(feature = "semantics")]
    //    {
    //        if let Some(model) = &self.params.semantic_model {
    //            let processed =
    //                model.infer(&text, &self.params.word_coloring.linguistic_coloring)?;
    //            debug!("processed: {processed:?}");
    //        }
    //    }
    //    let mut lines = Vec::new();
    //    for line in text.lines() {
    //        let line = params.colorize_text(line.to_string())?;
    //        lines.push(line);
    //    }
    //    self.processed_text = lines;
    //    Ok(())
    //}
    fn handle_events(&mut self) -> Result<(), ColcatError> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_press_event(key_event)?;
            }
            Event::Key(key_event) if key_event.kind == KeyEventKind::Release => {
                //TODO: handle key combos like control, shift, etc
                //self.handle_key_release_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    pub fn exit(&mut self) {
        self.state.display_state.exit = true;
    }
    pub fn display_text<'a>(&self, processed_text: &[ColoredCharacter]) -> Line<'a> {
        let spans = processed_text.iter().map(|colchar| {
            let ColoredCharacter {
                fg_color,
                bg_color,
                character,
            } = colchar;
            match self.state.text_state.params.coloring.coloring_scheme {
                ColoringScheme::ForegroundOnly => Span::styled(
                    String::from(*character),
                    Style::default().fg(fg_color.inner().into_ext()),
                ),
                ColoringScheme::ForegroundAndBackground => Span::styled(
                    String::from(*character),
                    Style::default().fg(fg_color.inner().into_ext()).bg(bg_color
                        .as_ref()
                        .map(|val| val.inner().into_ext())
                        .unwrap_or_default()),
                ),
                ColoringScheme::BackgroundOnly => Span::styled(
                    String::from(*character),
                    Style::default().bg(bg_color
                        .as_ref()
                        .map(|val| val.inner().into_ext())
                        .unwrap_or_default()),
                ),
            }
        });
        Line::from_iter(spans)
    }
    pub fn get_last_line(&self) -> i64 {
        i64::max(
            self.state.text_state.current_text.lines().count() as i64 - 1,
            0,
        )
    }
    //pub fn into_processed_text(self) -> Vec<Vec<ColoredCharacter>> {
    //    let TextState { processed_text, .. } = self.state.text_state;
    //    processed_text
    //}
}

impl<'a> IntoExt for &'a Vec<(csscolorparser::Color, char)> {
    type Output = Line<'a>;
    fn into_ext(self) -> Self::Output
    where
        Self: Sized,
    {
        let spans = self.iter().map(|(col, chara)| {
            Span::styled(String::from(*chara), Style::default().fg(col.into_ext()))
        });
        Line::from_iter(spans)
    }
}

impl<'a> IntoExt for &'a Vec<ColoredCharacter> {
    type Output = Line<'a>;
    fn into_ext(self) -> Self::Output
    where
        Self: Sized,
    {
        let spans = self.iter().map(|colchar| {
            let ColoredCharacter {
                fg_color,
                bg_color,
                character,
            } = colchar;
            if let Some(bg_color) = bg_color {
                Span::styled(
                    String::from(*character),
                    Style::default()
                        .fg(fg_color.inner().clone().into_ext())
                        .bg(bg_color.inner().clone().into_ext()),
                )
            } else {
                Span::styled(
                    String::from(*character),
                    Style::default().fg(fg_color.inner().clone().into_ext()),
                )
            }
        });
        Line::from_iter(spans)
    }
}

impl Widget for &RatatuiApp {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        //TODO: convert to match self.state.mode
        if self.state.needs_processing() {
            let pb = Gauge::default()
                .block(Block::bordered().title("Processing Text"))
                .gauge_style(Style::new().white().on_black().italic())
                .percent(0);
            pb.render(area, buf);
        } else {
            let processed_text = &self.state.text_state.processed_text;
            let iter = processed_text.iter().map(|val| self.display_text(val));
            let text = Text::from_iter(iter);
            let block = Block::default();
            //(y,x), evil
            let offset = (
                self.state
                    .display_state
                    .position
                    .line_number
                    .clamp(0, u16::MAX as i64) as u16,
                self.state
                    .display_state
                    .position
                    .line_idx
                    .clamp(0, u16::MAX as i64) as u16,
            );
            let para = Paragraph::new(text)
                .block(block)
                .wrap(self.state.display_state.wrap_mode)
                .scroll(offset);
            para.render(area, buf);
        }
    }
}
