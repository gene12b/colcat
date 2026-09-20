#[cfg(feature = "semantics")]
use candle_transformers::models::mimi::candle;
use color_eyre::eyre::bail;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum ColcatError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Renderer(#[from] RendererError),
    #[error("{0}")]
    Serde(#[from] ColcatSerdeError),
    #[error("Character {0:?} has no color mapping")]
    UnmappedCharacter(char),
    #[error("Character index {0:?} has no character it can be encoded as")]
    UnmappedCharacterIdx(usize),
    #[error("No usize index for {0:?} defined for custom indexing")]
    UnmappedCustomIndex(usize),
    #[error("{0}")]
    CssColorError(#[from] csscolorparser::ParseColorError),
    //#[error("{0}")]
    //ColorEyre(#[from] color_eyre::eyre::Report),
    #[cfg(feature = "semantics")]
    #[error("{0}")]
    Semantics(#[from] ColcatSemanticsError),
    ///("Type {0} does not have FromStr variant '{1}', acceptable values: {2:?}")
    #[error("Type {0} does not have FromStr variant '{1}', acceptable values: {2:?}")]
    FromStrParse(String, String, Vec<String>),
    #[error("Could not convert {0} to {1}")]
    BadConversion(String, String),
    #[error("{0}")]
    StrumParse(#[from] strum::ParseError),
}

#[cfg(feature = "semantics")]
#[derive(Debug, Error)]
pub enum ColcatSemanticsError {
    #[error("{0}")]
    Tokenizers(#[from] tokenizers::Error),
    #[error("{0}")]
    Candle(#[from] candle::Error),
    #[error("Tokenizer batch encoding failed: {0}")]
    TokenizerEncodeBatchFail(Box<dyn std::error::Error + Send + Sync>),
    #[error("Couldn't create tensor: {0}")]
    TensorAllocationFail(Box<dyn std::error::Error + Send + Sync>),
    #[error(
        "Need at least one phrase or entity set as target text to set as maximum semantic relevance"
    )]
    TargetTextCannotBeEmpty,
    #[error(
        "WordColoringKind:SemanticsTransformer selected but params for model were not supplied"
    )]
    SelectedButNoParams,
    #[error("Phrase {0} is missing a score in calculated semantics scores!")]
    PhraseMissingScore(String),
}

impl From<serde_json::Error> for ColcatError {
    fn from(value: serde_json::Error) -> Self {
        Self::from(ColcatSerdeError::from(value))
    }
}

impl From<toml::de::Error> for ColcatError {
    fn from(value: toml::de::Error) -> Self {
        Self::from(ColcatSerdeError::from(value))
    }
}

#[derive(Debug, Error)]
pub enum ColcatSerdeError {
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
    #[error("Unkown extension '{0:?}'")]
    Unknown(String),
}
pub trait UnwrapFallible {
    fn map_err_fallible(self) -> Result<(), color_eyre::Report>;
}

impl UnwrapFallible for Result<(), ColcatError> {
    fn map_err_fallible(self) -> Result<(), color_eyre::Report> {
        match self {
            Ok(()) => Ok(()),
            Err(err) => match err.fallible() {
                ColcatErrorFallibility::Fallible(err) => {
                    //print error
                    error!("{err:#?}");
                    Ok(())
                }
                ColcatErrorFallibility::Infallible(err) => {
                    //crash and burn no survivors
                    bail!(err);
                }
            },
        }
    }
}

impl ColcatError {
    pub fn fallible(self) -> ColcatErrorFallibility {
        match self {
            Self::Io(_) | Self::Renderer(_) => ColcatErrorFallibility::Fallible(self),
            other => ColcatErrorFallibility::Infallible(other),
        }
    }
}

pub enum ColcatErrorFallibility {
    Fallible(ColcatError),
    Infallible(ColcatError),
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Tried to transition to searching phase while already in searching phase")]
    TriedSearchWhileSearching,
}
