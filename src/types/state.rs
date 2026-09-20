use ratatui::widgets::Wrap;

use crate::{
    color::{
        ColoredCharacter,
        words::{WordColoringKind, WordColoringKindVanilla},
    },
    errors::ColcatError,
    renderer::KeyComboMode,
    types::params::ColcatParams,
};

#[derive(Debug)]
pub struct ColcatState {
    pub mode: ColcatMode,
    pub display_state: DisplayState,
    pub text_state: TextState,
}

impl ColcatState {
    pub fn new(params: ColcatParams, text: String) -> Result<Self, ColcatError> {
        let text_state = TextState::new(params, text)?;
        Ok(Self {
            mode: Default::default(),
            display_state: Default::default(),
            text_state,
        })
    }
    pub fn process_text(&mut self) -> Result<(), ColcatError> {
        self.text_state.process_text()
    }
    pub fn should_exit(&self) -> bool {
        self.display_state.exit
    }
    pub fn needs_processing(&self) -> bool {
        self.text_state.needs_processing()
    }
}

#[derive(Debug, Default)]
pub enum ColcatMode {
    ProcessingText {
        ratio_finished: f32,
    },
    #[default]
    Idle,
}

impl ColcatMode {
    pub fn new(config: &ColcatParams) -> Self {
        Self::default()
    }
}

impl From<&ColcatParams> for ColcatMode {
    fn from(value: &ColcatParams) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
pub struct DisplayState {
    pub position: CursorPosition,
    pub key_state: KeyComboMode,
    pub wrap_mode: Wrap,
    pub exit: bool,
}

#[derive(Debug, Default)]
pub struct CursorPosition {
    pub line_number: i64,
    pub line_idx: i64,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            position: Default::default(),
            key_state: Default::default(),
            wrap_mode: Wrap { trim: false },
            exit: false,
        }
    }
}

#[derive(Debug)]
pub struct TextState {
    pub needs_processing: bool,
    pub params: ColcatParams,
    pub current_text: String,
    pub processed_text: Vec<Vec<ColoredCharacter>>,
    #[cfg(feature = "semantics")]
    semantic_state: Option<semantics::SemanticState>,
}

impl TextState {
    pub fn new(params: ColcatParams, text: String) -> Result<Self, ColcatError> {
        #[cfg(feature = "semantics")]
        let semantic_state = if params.demands_semantics_processing() {
            use crate::errors::ColcatSemanticsError;
            Some(semantics::SemanticState::new(
                params
                    .semantic_model
                    .clone()
                    .ok_or(ColcatSemanticsError::SelectedButNoParams)?,
            )?)
        } else {
            None
        };

        Ok(Self {
            needs_processing: true,
            params,
            current_text: text,
            processed_text: Vec::new(),
            #[cfg(feature = "semantics")]
            semantic_state,
        })
    }
    pub fn process_text(&mut self) -> Result<(), ColcatError> {
        #[cfg(feature = "semantics")]
        {
            if self
                .semantic_state
                .as_mut()
                .is_some_and(|val| val.needs_processing())
            {
                self.semantic_state
                    .as_mut()
                    .map(|val| {
                        val.process_text(
                            &self.current_text,
                            &self.params.coloring.linguistic_coloring,
                        )
                    })
                    .transpose()?;
            }
        }
        let lines = self.colorize_text()?;
        self.processed_text = lines;
        self.needs_processing = false;
        Ok(())
    }
    pub fn needs_processing(&self) -> bool {
        let mut needs_processing = false;
        #[cfg(feature = "semantics")]
        {
            use crate::types::state::semantics::SemanticState;

            needs_processing |= self
                .semantic_state
                .as_ref()
                .is_some_and(SemanticState::needs_processing);
        }
        needs_processing |= self.needs_processing;
        needs_processing
    }
    pub fn colorize_text(&mut self) -> Result<Vec<Vec<ColoredCharacter>>, ColcatError> {
        let params = &self.params;
        match &self.params.coloring.word_coloring_kind {
            #[cfg(feature = "semantics")]
            WordColoringKind::SemanticsTransformer => {
                use crate::{color::SplitColoredCharacterLines, errors::ColcatSemanticsError};
                let processed_semantics = &self
                    .semantic_state
                    .as_ref()
                    .ok_or(ColcatSemanticsError::SelectedButNoParams)
                    .map_err(ColcatError::from)?
                    .processed_semantics;
                let characters = params
                    .colorize_semantics(self.current_text.clone(), processed_semantics)?
                    .split_char('\n');
                Ok(characters)
            }
            other => {
                let mut lines = Vec::new();
                let word_coloring_kind_vanilla = WordColoringKindVanilla::try_from(other.clone())?;
                for line in self.current_text.lines() {
                    let line =
                        params.colorize_text(line.to_string(), &word_coloring_kind_vanilla)?;
                    lines.push(line);
                }
                Ok(lines)
            }
        }
    }
}

#[cfg(feature = "semantics")]
mod semantics {
    use crate::color::semantics::{ProcessedSemantics, SemanticsModel, SemanticsParamsSerde};
    use crate::color::words::LinguisticColoring;
    use crate::errors::{ColcatError, ColcatSemanticsError};

    #[derive(Debug)]
    pub struct SemanticState {
        pub model: SemanticsModel,
        pub processed_semantics: Vec<ProcessedSemantics>,
        needs_processing: bool,
    }

    impl SemanticState {
        pub fn new(params: SemanticsParamsSerde) -> Result<Self, ColcatError> {
            let model = SemanticsModel::new(params)?;
            Ok(Self {
                model,
                processed_semantics: Vec::new(),
                needs_processing: true,
            })
        }
        pub fn process_text(
            &mut self,
            text: &str,
            split_mode: &LinguisticColoring,
        ) -> Result<(), ColcatSemanticsError> {
            let processed = self.model.infer(text, split_mode)?;
            self.processed_semantics = processed;
            self.needs_processing = false;
            Ok(())
        }
        pub fn needs_processing(&self) -> bool {
            self.needs_processing
        }
    }
}

//self.semantic_state
//    .as_mut()
//    .ok_or(ColcatSemanticsError::SelectedButNoParams)
//    .map_err(ColcatError::from)?
//    .process_text(
//        &self.current_text,
//        &self.params.coloring.linguistic_coloring,
//    )?;
