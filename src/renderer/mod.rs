#[cfg(feature = "pdf")]
pub mod pdf;

use std::{str::FromStr, time::Duration};

use crossterm::event::{self, Event, KeyEventKind};
use itertools::Itertools;
use palette::num::Clamp;
use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Gauge, Paragraph, Widget},
};
use ratatui_notifications::{Anchor, Animation, Level, Notification, Notifications};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator, VariantNames};
use tracing::debug;

use crate::{
    color::{ColoredCharacter, ColoringScheme, SplitColoredCharacterLines},
    errors::{ColcatError, UnwrapFallible},
    ratatui::event_handle::RatatuiKeyEventHandle,
    types::{
        IntoExt,
        params::ColcatParams,
        state::{
            ColcatState,
            //Notification
        },
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
    pub notifications: Notifications,
}

#[derive(Debug, Default)]
pub enum KeyComboMode {
    #[default]
    View,
    //Insert,
    Go,
    Search(SearchModePhase),
    Command(CommandModePhase),
}

#[derive(Debug)]
pub enum SearchModePhase {
    Typing(String),
    Searching { hits: Vec<usize>, hit_idx: usize },
}

#[derive(Debug)]
pub enum CommandModePhase {
    Typing(String),
}

impl Default for CommandModePhase {
    fn default() -> Self {
        Self::Typing(String::new())
    }
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
            ':' => Self::Command(Default::default()),
            _ => Self::View,
        }
    }
}

impl RatatuiApp {
    pub fn new(text: String, params: ColcatParams) -> Result<Self, ColcatError> {
        Ok(Self {
            state: ColcatState::new(params, text)?,
            notifications: Notifications::new(),
        })
    }
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), color_eyre::eyre::Report> {
        debug!("state: {:#?}", self.state);
        while !self.state.should_exit() {
            //debug!("main loop");
            self.notifications.tick(Duration::from_millis(16));
            terminal.draw(|frame| {
                self.draw(frame);
                self.notifications.render(frame, frame.area());
            })?;
            //debug!("main loop draw");

            if self.state.needs_processing() {
                debug!("needs_processing");
                self.state
                    .process_text()
                    .map_err_fallible_notify(&mut self.notifications)?;
                debug!("processed");
                self.state.text_state.needs_processing = false;
                //I think this is worth an immediate redraw
                self.notifications.tick(Duration::from_millis(16));
                terminal.draw(|frame| {
                    self.draw(frame);
                    self.notifications.render(frame, frame.area());
                })?;
            }
            self.handle_events()
                .map_err_fallible_notify(&mut self.notifications)?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<(), ColcatError> {
        let timeout = Duration::from_millis(16);
        if event::poll(timeout)? {
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
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    pub fn exit(&mut self) {
        self.state.display_state.exit = true;
    }
    pub fn show_notification(
        &mut self,
        notification_title: String,
        notification_text: String,
    ) -> Result<(), ColcatError> {
        //self.state.notification = Some(Notification::new(notification_title, notification_text));
        let notif = Notification::new(notification_text)
            .title(notification_title)
            .level(Level::Info)
            .anchor(Anchor::TopCenter)
            .animation(Animation::Slide)
            .build()?;
        self.notifications.add(notif)?;
        Ok(())
    }
    pub fn display_text<'a>(&self, processed_text: &[ColoredCharacter]) -> Vec<Line<'a>> {
        let mut ret_vec = Vec::new();
        let split = processed_text.to_vec().split_char('\n');
        for processed_text in split {
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
            ret_vec.push(Line::from_iter(spans))
        }
        ret_vec
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
                        .fg((*fg_color.inner()).into_ext())
                        .bg((*bg_color.inner()).into_ext()),
                )
            } else {
                Span::styled(
                    String::from(*character),
                    Style::default().fg((*fg_color.inner()).into_ext()),
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
        let block = Block::default();
        //TODO: convert to match self.state.mode
        if self.state.needs_processing() {
            let shrunken_area = Rect::new(
                area.x + 6,
                area.y + 6,
                (area.width - 4 * 3).clamp_min(1),
                (area.height - 4 * 3).clamp_min(1),
            );
            let pb = Gauge::default()
                .block(Block::bordered().title("Processing Text"))
                .gauge_style(Style::new().white().on_black().italic())
                .percent(0);
            pb.render(shrunken_area, buf);
        } else {
            let processed_text = &self.state.text_state.processed_text;
            let iter = self.display_text(processed_text);
            let text = Text::from(iter);
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
        if let KeyComboMode::Command(phase) = &self.state.display_state.key_state {
            match phase {
                CommandModePhase::Typing(text) => {
                    let bottom_text = format!(":{text}");
                    let para = Paragraph::new(bottom_text).style(Style::new().white().on_blue());
                    let bottom_line = Rect::new(area.x, area.bottom() - 1, area.width, 1);
                    para.render(bottom_line, buf);
                }
            }
        }
        if let Some(overlaying_message) = &self.state.display_state.overlaying_message {
            let para = Paragraph::new(overlaying_message.as_str()); //.style(Style::new());
            let notification_area = Rect::new(
                area.x,
                area.y,
                (area.width - 3).clamp_min(1),
                (area.height - 3).clamp_min(1),
            );
            para.render(notification_area, buf);
        }
    }
}
