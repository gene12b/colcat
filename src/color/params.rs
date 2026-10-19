use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use vecstat::{StatisticsUnorderedF32, calc_traits::Zeroable, partial::StatisticsOrderedPartial};

use crate::{
    color::{
        ColorColcat, ColorWrap, ColoredCharacter, ColoringScheme,
        palettes::ColorPalette,
        words::{
            LinguisticColoring, WordColoringKind, WordColoringKindSerde, WordColoringKindVanilla,
        },
    },
    errors::ColcatError,
    types::{indexing::IndexingScale, params::UtfRenderMode},
};

//#[skip_serializing_none]
#[derive(Debug, Default)]
pub struct ColorParams {
    pub linguistic_coloring: LinguisticColoring,
    pub word_coloring_kind: WordColoringKind,
    pub coloring_scheme: ColoringScheme,
    pub palette: ColorPalette,
    extra_interpolation: ExtraInterpolationConfig,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct ColorParamsSerde {
    pub linguistic_coloring: LinguisticColoring,
    word_coloring_kind: WordColoringKindSerde,
    pub coloring_scheme: ColoringScheme,
    palette: ColorPalette,
    extra_interpolation: Option<ExtraInterpolationConfig>,
}

impl TryFrom<ColorParamsSerde> for ColorParams {
    type Error = ColcatError;
    fn try_from(value: ColorParamsSerde) -> Result<Self, Self::Error> {
        let ColorParamsSerde {
            linguistic_coloring,
            word_coloring_kind,
            coloring_scheme,
            palette,
            extra_interpolation,
        } = value;
        Ok(ColorParams {
            linguistic_coloring,
            word_coloring_kind: WordColoringKind::try_from(word_coloring_kind)?,
            coloring_scheme,
            palette,
            extra_interpolation: extra_interpolation.unwrap_or_default(),
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct ExtraInterpolationConfig {
    fg_interpolation: Option<ColorInterpolation>,
    bg_interpolation: Option<ColorInterpolation>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct ColorInterpolation {
    amount: f32,
    target_color: ColorColcat,
}

impl ColorParams {
    pub fn colorize_text(
        &self,
        text: String,
        indexing: &IndexingScale,
        utf_render_mode: &[UtfRenderMode],
        word_coloring_kind: &WordColoringKindVanilla,
    ) -> Result<Vec<ColoredCharacter>, ColcatError> {
        let split_text = self.linguistic_coloring.split_text(text);
        let colors = split_text
            .into_iter()
            .map(|val| {
                self.colorize_string(val.as_str(), indexing, utf_render_mode, word_coloring_kind)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ret_vec = self.linguistic_coloring.join_colchars(colors);
        Ok(ret_vec)
    }

    //let colors = match &self.linguistic_coloring {
    //    LinguisticColoring::Words => {
    //        let words = text.split_whitespace();
    //
    //        words
    //            .map(|word| {
    //                self.colorize_string(word, indexing, utf_render_mode, word_coloring_kind)
    //            })
    //            .collect::<Result<Vec<_>, _>>()?
    //    }
    //    LinguisticColoring::Sentences { delimiter } => {
    //        let sentences = text.split(*delimiter);
    //
    //        sentences
    //            .filter(|val| !val.is_empty())
    //            .map(|val| {
    //                self.colorize_string(val, indexing, utf_render_mode, word_coloring_kind)
    //            })
    //            .collect::<Result<Vec<_>, _>>()?
    //    }
    //    LinguisticColoring::Characters => {
    //        let characters = text.chars().map(|char_val| char_val.to_string());
    //
    //        characters
    //            .map(|val| {
    //                self.colorize_string(
    //                    val.as_str(),
    //                    indexing,
    //                    utf_render_mode,
    //                    word_coloring_kind,
    //                )
    //            })
    //            .collect::<Result<Vec<_>, _>>()?
    //    }
    //};

    fn colorize_string(
        &self,
        word: &str,
        indexing: &IndexingScale,
        utf_render_mode: &[UtfRenderMode],
        word_coloring_kind: &WordColoringKindVanilla,
        //#[cfg(feature = "semantics")] processed_semantic: Option<ProcessedSemantics>,
    ) -> Result<Vec<ColoredCharacter>, ColcatError> {
        let mut colorchars = word
            .chars()
            .map(|character| {
                self.palette
                    .colorize_char(character, indexing, utf_render_mode)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let colors = colorchars
            .clone()
            .into_iter()
            .map(|val| val.fg_color)
            .collect::<Vec<_>>();
        let bg_color = match word_coloring_kind {
            WordColoringKindVanilla::Average => colors.avg().unwrap(),
            WordColoringKindVanilla::Darkest => colors.try_min_stat().unwrap().unwrap().clone(),
            WordColoringKindVanilla::Brightest => colors.try_max_stat().unwrap().unwrap().clone(),
            WordColoringKindVanilla::Uniform(val) => val.clone(),
        };
        colorchars
            .iter_mut()
            .for_each(|val| val.bg_color = Some(bg_color.clone()));
        Ok(colorchars)
    }
}

//#[cfg(feature = "semantics")]
//WordColoringKind::SemanticsTransformer => {
//    todo!("impl ColorParams::colorize_string WordColoringKind::SemanticsTransformer");
//}
