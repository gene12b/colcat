use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIs, EnumString, VariantNames};

//#[cfg(feature = "semantics")]
//use crate::color::semantics::{ProcessedSemantics, SemanticsModel, SemanticsModelSerde};
use crate::{
    color::{ColorColcat, ColoredCharacter},
    errors::ColcatError,
};

#[derive(Debug, Default, EnumIs, AsRefStr, Clone)]
pub enum WordColoringKind {
    #[cfg(feature = "semantics")]
    SemanticsTransformer,
    Average,
    #[default]
    Darkest,
    Brightest,
    Uniform(ColorColcat),
}

#[derive(Debug, Default, EnumIs, EnumString)]
pub enum WordColoringKindVanilla {
    Average,
    #[default]
    Darkest,
    Brightest,
    Uniform(ColorColcat),
}

impl TryFrom<WordColoringKind> for WordColoringKindVanilla {
    type Error = ColcatError;
    fn try_from(value: WordColoringKind) -> Result<Self, Self::Error> {
        match value {
            #[cfg(feature = "semantics")]
            WordColoringKind::SemanticsTransformer => Err(ColcatError::BadConversion(
                "WordColoringKind".to_string(),
                "WordColoringKindVanilla".to_string(),
            )),
            WordColoringKind::Uniform(val) => Ok(WordColoringKindVanilla::Uniform(val)),
            other => Ok(WordColoringKindVanilla::from_str(other.as_ref())?),
        }
    }
}

//{
//    //model: Box<SemanticsModel>,
//    processed: Vec<ProcessedSemantics>,
//},

impl TryFrom<WordColoringKindSerde> for WordColoringKind {
    type Error = ColcatError;
    fn try_from(value: WordColoringKindSerde) -> Result<Self, Self::Error> {
        Ok(match value {
            #[cfg(feature = "semantics")]
            WordColoringKindSerde::SemanticsTransformer => WordColoringKind::SemanticsTransformer,
            WordColoringKindSerde::Average => WordColoringKind::Average,
            WordColoringKindSerde::Darkest => WordColoringKind::Darkest,
            WordColoringKindSerde::Brightest => WordColoringKind::Brightest,
            WordColoringKindSerde::Uniform(val) => WordColoringKind::Uniform(val),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Default, VariantNames)]
pub enum WordColoringKindSerde {
    #[cfg(feature = "semantics")]
    SemanticsTransformer,
    //{
    //    model_config: SemanticsModelSerde,
    //},
    Average,
    #[default]
    Darkest,
    //consider adding a background interpolation param
    //DarkToZeroInterp {
    //    interpolation: f32,
    //},
    Brightest,
    Uniform(ColorColcat),
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub enum LinguisticColoring {
    #[default]
    ///Morphology
    Words,
    ///Semantics
    Sentences { delimiter: char },
    ///No linguistic element, all characters are treated the same
    Characters,
}

//impl LinguisticColoring {
//    pub fn split_and_colorize_text(
//        &self,
//        text: String,
//    ) -> Result<Vec<Vec<ColoredCharacter>>, ColcatError> {
//        match self {
//            LinguisticColoring::Words => {
//                let words = text.split_whitespace();
//
//                words
//                    .map(|word| {
//                        self.colorize_string(word, indexing, utf_render_mode, word_coloring_kind)
//                    })
//                    .collect::<Result<Vec<_>, _>>()?
//            }
//            LinguisticColoring::Sentences { delimiter } => {
//                let sentences = text.split(*delimiter);
//
//                sentences
//                    .filter(|val| !val.is_empty())
//                    .map(|val| {
//                        self.colorize_string(val, indexing, utf_render_mode, word_coloring_kind)
//                    })
//                    .collect::<Result<Vec<_>, _>>()?
//            }
//            LinguisticColoring::Characters => {
//                let characters = text.chars().map(|char_val| char_val.to_string());
//
//                characters
//                    .map(|val| {
//                        self.colorize_string(
//                            val.as_str(),
//                            indexing,
//                            utf_render_mode,
//                            word_coloring_kind,
//                        )
//                    })
//                    .collect::<Result<Vec<_>, _>>()?
//            }
//        }
//    }
//}
