use ratatui::widgets::Wrap;

#[cfg(feature = "semantics")]
use crate::errors::ColcatSemanticsError;
use crate::{
    color::{
        ColoredCharacter,
        words::{WordColoringKind, WordColoringKindVanilla},
    },
    errors::ColcatError,
    renderer::KeyComboMode,
    types::params::ColcatParams,
};

//#[derive(Debug)]
//pub struct Notification {
//    title: String,
//    text: String,
//}

//impl Notification {
//    pub fn new(title: String, text: String) -> Self {
//        Self { title, text }
//    }
//}

#[derive(Debug)]
pub struct ColcatState {
    pub mode: ColcatMode,
    pub display_state: DisplayState,
    pub text_state: TextState,
    pub current_progress: u16,
    //pub notification: Option<Notification>,
    //pub tasks: Vec<JoinHandle<Result<(), ColcatErrorSend>>>,
    //pub message_stack: Vec<(TaskKind, InternalMessage)>,
    //pub task_comms: HashMap<TaskKind, mpsc::Receiver<Result<InternalMessage, ColcatErrorSend>>>,
}

impl ColcatState {
    pub fn new(params: ColcatParams, text: String) -> Result<Self, ColcatError> {
        let text_state = TextState::new(params, text)?;
        Ok(Self {
            mode: Default::default(),
            display_state: Default::default(),
            current_progress: 0,
            //notification: None,
            //tasks: Default::default(),
            //message_stack: Default::default(),
            //task_comms: Default::default(),
            text_state,
        })
    }
    pub fn set_text(&mut self, text: String) {
        self.text_state.set_text(text);
    }
    //pub async fn process_text(&mut self) -> Result<(), ColcatError> {
    //    self.text_state.process_text()
    //}
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

//pub trait ColcatStateAtomic {
//    async fn new(params: ColcatParams, text: String) -> Result<Self, ColcatError>
//    where
//        Self: Sized;
//    async fn set_text(&self, text: String);
//    async fn process_text(&mut self) -> Result<(), ColcatError>;
//    async fn should_exit(&self) -> bool;
//    async fn needs_processing(&self) -> bool;
//}
//
//impl ColcatStateAtomic for Arc<Mutex<ColcatState>> {
//    async fn set_text(&self, text: String) {
//        let mut lock = self.lock().await;
//        lock.set_text(text);
//    }
//    async fn should_exit(&self) -> bool {
//        let lock = self.lock().await;
//        lock.should_exit()
//    }
//    async fn process_text(&mut self) -> Result<(), ColcatError> {
//        let mut lock = self.lock().await;
//        //lock.process_text().await
//        lock.process_text()
//    }
//    async fn needs_processing(&self) -> bool {
//        let lock = self.lock().await;
//        lock.needs_processing()
//    }
//    async fn new(params: ColcatParams, text: String) -> Result<Self, ColcatError>
//    where
//        Self: Sized,
//    {
//        Ok(Arc::new(Mutex::new(ColcatState::new(params, text)?)))
//    }
//}

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
    pub overlaying_message: Option<String>,
}

impl DisplayState {
    pub fn escape_pressed(&mut self) {
        self.key_state = KeyComboMode::View;
        self.overlaying_message = None;
    }
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
            overlaying_message: None,
        }
    }
}

#[derive(Debug)]
pub struct TextState {
    pub needs_processing: bool,
    pub params: ColcatParams,
    pub current_text: String,
    pub processed_text: Vec<ColoredCharacter>,
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
    pub fn colorize_text(&mut self) -> Result<Vec<ColoredCharacter>, ColcatError> {
        let params = &self.params;
        match &self.params.coloring.word_coloring_kind {
            #[cfg(feature = "semantics")]
            WordColoringKind::SemanticsTransformer => {
                use crate::errors::ColcatSemanticsError;
                let processed_semantics = &self
                    .semantic_state
                    .as_ref()
                    .ok_or(ColcatSemanticsError::SelectedButNoParams)
                    .map_err(ColcatError::from)?
                    .processed_semantics;
                let characters =
                    params.colorize_semantics(self.current_text.clone(), processed_semantics)?;
                Ok(characters)
            }
            other => {
                let mut lines = Vec::new();
                let word_coloring_kind_vanilla = WordColoringKindVanilla::try_from(other.clone())?;
                for line in self.current_text.lines() {
                    let mut line =
                        params.colorize_text(line.to_string(), &word_coloring_kind_vanilla)?;
                    lines.append(&mut line);
                }
                Ok(lines)
            }
        }
    }
    pub fn set_text(&mut self, text: String) {
        self.current_text = text;
    }
    #[cfg(feature = "semantics")]
    pub fn set_semantics_target_string(
        &mut self,
        new_text: String,
    ) -> Result<(), ColcatSemanticsError> {
        use crate::errors::ColcatSemanticsError;

        let sem_state = self
            .semantic_state
            .as_mut()
            .ok_or(ColcatSemanticsError::Uninitialized)?;

        sem_state.model.set_target_text(new_text);
        sem_state.set_needs_processing(true);

        Ok(())
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
        pub fn set_needs_processing(&mut self, needs_processing: bool) {
            self.needs_processing = needs_processing;
        }
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
