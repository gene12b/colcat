use std::{
    fs::File,
    io::{BufReader, Read},
    ops::Range,
    path::{Path, PathBuf},
};

use clap::Parser;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
//use tracing::debug;

#[cfg(feature = "semantics")]
use crate::color::semantics::{ProcessedSemantics, SemanticsParamsSerde};
use crate::{
    color::{
        ColoredCharacter,
        params::{ColorParams, ColorParamsSerde},
        words::WordColoringKindVanilla,
    },
    errors::{ColcatError, ColcatSerdeError},
    renderer::RendererDiscriminants,
    types::{indexing::IndexingScale, iter::VecEmptyOr},
};

/*
 * example config.json
{
    "colors": {
        "palette": {"Uniform":"#FFFFFF"},
        "background_color": "#272822"
    }
}
* */

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct ColcatParamsSerde {
    indexing: Option<IndexingScale>,
    utf_render_mode: Option<Vec<UtfRenderMode>>,
    word_coloring: Option<ColorParamsSerde>,
    #[cfg(feature = "semantics")]
    pub semantic_model_info: Option<SemanticsParamsSerde>,
}

impl ColcatParamsSerde {
    pub fn try_from_path(path: impl AsRef<Path>) -> Result<Self, ColcatError> {
        let mut reader = BufReader::new(File::open(path.as_ref())?);
        let val = serde_json::from_reader(&mut reader)?;
        Ok(val)
    }
}

#[derive(Debug)]
pub struct ColcatParams {
    pub indexing: IndexingScale,
    pub utf_render_mode: Vec<UtfRenderMode>,
    pub coloring: ColorParams,
    #[cfg(feature = "semantics")]
    pub semantic_model: Option<SemanticsParamsSerde>,
}

impl ColcatParams {
    pub fn colorize_text(
        &self,
        text: String,
        word_coloring_kind: &WordColoringKindVanilla,
    ) -> Result<Vec<ColoredCharacter>, ColcatError> {
        let mut text = self.coloring.colorize_text(
            text,
            &self.indexing,
            &self.utf_render_mode,
            word_coloring_kind,
        )?;
        //TODO: interpolate ColoredCharacters to fg and background values
        Ok(text)
    }

    #[cfg(feature = "semantics")]
    pub fn colorize_semantics(
        &self,
        text: String,
        processed_semantics: &[ProcessedSemantics],
    ) -> Result<Vec<ColoredCharacter>, ColcatError> {
        //TODO: expand this to allow more than just coloring based on the first phrase

        let mut ret_chars = Vec::new();
        use tracing::debug;

        use crate::errors::ColcatSemanticsError;
        let target_semantic = processed_semantics
            .first()
            .ok_or(ColcatSemanticsError::TargetTextCannotBeEmpty)
            .map_err(ColcatError::from)?;
        //TODO: use predefined delimiter
        let sentences_raw = text.split(".");
        //debug!("sentences_raw: {sentences_raw:?}");
        let sentences = text
            .split(".")
            .map(str::trim)
            //.map(|val| val.replace("\n", ""))
            .map(String::from)
            .filter(|val| !val.is_empty());
        for (sentence, sentence_raw) in sentences.zip(sentences_raw) {
            debug!("sentence_raw: {sentence_raw:?}");
            use vecstat::calc_traits::Zeroable;

            use crate::color::ColorColcat;

            let fractional = target_semantic.phrase_score_map.get(&sentence).ok_or(
                ColcatSemanticsError::PhraseMissingScore(sentence.to_string()),
            )?;
            let color = self.coloring.palette.colorize_idx(*fractional as f64)?;
            let mut ret_sentence = self.coloring.colorize_text(
                sentence_raw.to_string(),
                &self.indexing,
                &self.utf_render_mode,
                &WordColoringKindVanilla::Uniform(color),
            )?;
            ret_sentence.push(ColoredCharacter::new(ColorColcat::zero(), None, '.'));
            //ret_sentence.push(ColoredCharacter::new(ColorColcat::zero(), None, '\n'));
            ret_chars.append(&mut ret_sentence);
        }
        Ok(ret_chars)
    }
    #[cfg(feature = "semantics")]
    pub fn demands_semantics_processing(&self) -> bool {
        self.coloring.word_coloring_kind.is_semantics_transformer()
    }
}

///How do you want to map color palettes to
///the character space provided?
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct UtfRenderMode {
    charset: CharSet,
    exclusivity: Option<Exclusivity>,
}

impl UtfRenderMode {
    pub fn new(charset: CharSet, exclusivity: Option<Exclusivity>) -> Self {
        Self {
            charset,
            exclusivity,
        }
    }
    pub fn contains(&self, character: char) -> bool {
        self.charset.contains(character)
    }
}

const MAX_ASCII: usize = 0xff;
const MAX_UTF8: usize = char::MAX as usize;
//const MAX_UTF16:

//fn char_range(val: usize) -> Result<CharSet, ColcatError> {
//    match val {
//        0..MAX_ASCII => Ok(CharSet::AsciiFull),
//        MAX_ASCII..MAX_UTF8 => Ok(CharSet::Utf8),
//        other => Err(ColcatError::UnmappedCharacterIdx(other)),
//    }
//}

impl UtfRenderMode {
    pub fn calc_idx(&self, character: char, indexing: &IndexingScale) -> Result<f64, ColcatError> {
        //let char_range = char_range(character as usize)?;
        let char_range = &self.charset;
        let idx = self
            .exclusivity
            .clone()
            .map(|val| val.idx(character, char_range))
            .unwrap_or(0.0);
        //debug!("char: {character} in charset: {char_range:?} -> idx {idx}");
        indexing.scale(idx, 0.0, 1.0)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
pub enum CharSet {
    AsciiUppercase,
    AsciiLowercase,
    AsciiDigits,
    AsciiPrintable,
    AsciiExtended,
    AsciiFull,
    Utf8,
    //Utf16,
}

impl CharSet {
    pub fn contains(&self, character: char) -> bool {
        //debug!(
        //    "charset: {self:?}, character: {character} as usize {}, idx_range: {:?}",
        //    character as usize,
        //    self.idx_range()
        //);
        self.idx_range().contains(&(character as usize))
    }
    pub fn idx_range(&self) -> Range<usize> {
        match self {
            CharSet::AsciiUppercase => 65..91,
            CharSet::AsciiLowercase => 97..123,
            CharSet::AsciiDigits => 48..58,
            CharSet::AsciiPrintable => 32..128,
            CharSet::AsciiExtended => 128..MAX_ASCII,
            CharSet::AsciiFull => 0..MAX_ASCII,
            CharSet::Utf8 => MAX_ASCII..MAX_UTF8,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, JsonSchema)]
pub enum Exclusivity {
    #[default]
    Repeat,
    Join,
}

impl Exclusivity {
    pub fn idx(&self, character: char, charset: &CharSet) -> f64 {
        let float_char = character as u32 as f64;
        match charset {
            CharSet::AsciiPrintable
            | CharSet::AsciiExtended
            | CharSet::AsciiUppercase
            | CharSet::AsciiLowercase
            | CharSet::AsciiDigits
            | CharSet::AsciiFull => match self {
                Exclusivity::Repeat => {
                    (character as usize - charset.idx_range().start) as f64
                        / (charset.idx_range().len() as f64)
                }
                Exclusivity::Join => float_char / MAX_UTF8 as f64,
            },
            //CharSet::AsciiFull => match self {
            //    Exclusivity::Repeat => float_char / MAX_ASCII as f64,
            //    Exclusivity::Join => float_char / MAX_UTF8 as f64,
            //},
            CharSet::Utf8 => match self {
                Exclusivity::Repeat => (character as usize - MAX_ASCII) as f64 / MAX_UTF8 as f64,
                Exclusivity::Join => float_char / MAX_UTF8 as f64,
            },
        }
    }
}

impl TryFrom<ColcatParamsSerde> for ColcatParams {
    type Error = ColcatError;
    fn try_from(value: ColcatParamsSerde) -> Result<Self, Self::Error> {
        Ok(Self {
            //palette: value.palette.unwrap_or_default(),
            indexing: value.indexing.unwrap_or_default(),
            //salient_words: value.salient_words.unwrap_or_default(),
            utf_render_mode: value.utf_render_mode.unwrap_or_default().empty_or(|| {
                //colorize uppercase and lowercase letters equally
                vec![
                    UtfRenderMode::new(CharSet::AsciiLowercase, Some(Exclusivity::Repeat)),
                    UtfRenderMode::new(CharSet::AsciiUppercase, Some(Exclusivity::Repeat)),
                ]
            }),
            coloring: value.word_coloring.unwrap_or_default().try_into()?,
            #[cfg(feature = "semantics")]
            semantic_model: value.semantic_model_info, //.map(SemanticsModel::new)
                                                       //.transpose()?,
        })
    }
}

impl TryFrom<PathBuf> for ColcatParamsSerde {
    type Error = ColcatError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let mut file = File::open(&value)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let config = match value.extension().and_then(|val| val.to_str()).unwrap() {
            "json" => serde_json::from_slice(&buf)?,
            "toml" => toml::from_slice(&buf)?,
            other => {
                return Err(ColcatSerdeError::Unknown(other.to_string()).into());
            }
        };
        Ok(config)
    }
}

//This will be used in release as LSP
#[derive(Debug, Parser)]
pub struct ColcatArgs {}

//This is for Tui rendering of text
#[derive(Debug, Parser)]
pub struct ColcatTuiArgs {
    #[arg(num_args = 0..=1)]
    pub input_file: Option<PathBuf>,
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    #[arg(short, long)]
    pub renderer: Option<RendererDiscriminants>,
    #[arg(short, long)]
    pub generate_config_schema: Option<PathBuf>,
}
