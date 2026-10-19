use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    color::{ColorColcat, ColorWrap, ColoredCharacter, palettes::ColorPalette::Uniform},
    errors::ColcatError,
    types::{indexing::IndexingScale, params::UtfRenderMode},
};

//defines fg and background colors
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub enum ColorPalette {
    #[default]
    ///Default because it's everywhere.
    Monokai,
    ///Maximum visual overload go: truly brutal on the eyes.
    ///Only for the most insane speedreaders who want maximum
    ///information but can handle the mental exhaustion.
    LinearRGB,
    //One of two mentally safe color pallettes for reading
    //Inferno,
    Veridis,
    ///A special color dedicated to each character
    PerCharacter {
        char_map: HashMap<char, ColorColcat>,
        ///If character doesn't exist in char_map when rendering, what color
        ///do you render it as?
        fallback_color: Option<ColorColcat>,
    },
    ///Will interpolate colors equidistantly between provided list of colors
    ///in the order they are provided.
    ColorVec(Vec<ColorColcat>),
    Uniform(ColorColcat),
}

impl ColorPalette {
    //pub fn colorize_text(
    //    &self,
    //    text: String,
    //    indexing: &IndexingScale,
    //    utf_render_mode: &[UtfRenderMode],
    //) -> Result<Vec<(ColorColcat, char)>, ColcatError> {
    //    text.chars()
    //        .map(|character| {
    //            self.colorize_char(character, indexing, utf_render_mode)
    //                .map(|color| (color, character))
    //        })
    //        .collect::<Result<Vec<_>, _>>()
    //}
    pub fn colorize_char(
        &self,
        character: char,
        indexing: &IndexingScale,
        utf_render_mode: &[UtfRenderMode],
    ) -> Result<ColoredCharacter, ColcatError> {
        match self {
            Uniform(col) => Ok(col.clone()),
            ColorPalette::Monokai => Self::calc_color(
                &MONOKAI_FG
                    .into_iter()
                    .map(csscolorparser::parse)
                    .map(|val| val.map(Into::into))
                    .collect::<Result<Vec<_>, _>>()?,
                character,
                indexing,
                utf_render_mode,
            ),
            ColorPalette::Veridis => Self::calc_color(
                &VERIDIS_FG
                    .into_iter()
                    .map(csscolorparser::parse)
                    .map(|val| val.map(Into::into))
                    .collect::<Result<Vec<_>, _>>()?,
                character,
                indexing,
                utf_render_mode,
            ),
            //Prepare your eyes
            ColorPalette::LinearRGB => {
                let idx = utf_render_mode
                    .iter()
                    .find(|render_mode| render_mode.contains(character))
                    .map(|render_mode| render_mode.calc_idx(character, indexing))
                    .unwrap_or(Ok(0.0))?;
                let fractional = idx * 360.0;
                Ok(ColorColcat::from_hsva(fractional as f32, 1.0, 1.0, 1.0))
            }
            ColorPalette::PerCharacter {
                char_map,
                fallback_color,
            } => Ok(char_map
                .get(&character)
                .unwrap_or(&fallback_color.clone().unwrap_or_default())
                .clone()),
            ColorPalette::ColorVec(stack) => {
                Self::calc_color(stack, character, indexing, utf_render_mode)
            }
        }
        .map(|fg_color| ColoredCharacter::new(fg_color, None, character))
    }
    fn calc_color(
        color_stack: &[ColorColcat],
        character: char,
        indexing: &IndexingScale,
        utf_render_mode: &[UtfRenderMode],
    ) -> Result<ColorColcat, ColcatError> {
        let idx = utf_render_mode
            .iter()
            .find(|render_mode| render_mode.contains(character))
            .map(|render_mode| render_mode.calc_idx(character, indexing))
            .unwrap_or(Ok(0.0))?;
        Self::calc_color_idx(color_stack, idx)
    }
    fn calc_color_idx(color_stack: &[ColorColcat], idx: f64) -> Result<ColorColcat, ColcatError> {
        let fractional = idx * (color_stack.len() as f64);
        let color_line_init = fractional as usize;
        let fractional_diff = fractional - color_line_init as f64;
        let color_line_end = color_line_init + 1;
        let ret_color = if color_line_end == color_stack.len() {
            //OOB, so return last color
            color_stack[color_line_init].clone()
        } else {
            color_stack[color_line_init]
                .interpolate_rgb(&color_stack[color_line_end], fractional_diff as f32)
        };
        Ok(ret_color)
    }
    ///idx needs to be in 0..1
    pub fn colorize_idx(&self, idx: f64) -> Result<ColorWrap, ColcatError> {
        match self {
            ColorPalette::Monokai => Self::calc_color_idx(
                &MONOKAI_FG
                    .into_iter()
                    .map(csscolorparser::parse)
                    .map(|val| val.map(Into::into))
                    .collect::<Result<Vec<_>, _>>()?,
                idx,
            ),
            ColorPalette::Veridis => Self::calc_color_idx(
                &VERIDIS_FG
                    .into_iter()
                    .map(csscolorparser::parse)
                    .map(|val| val.map(Into::into))
                    .collect::<Result<Vec<_>, _>>()?,
                idx,
            ),
            ColorPalette::LinearRGB => {
                Ok(ColorColcat::from_hsva((idx * 360.0) as f32, 1.0, 1.0, 1.0))
            }

            ColorPalette::ColorVec(color_stack) => Self::calc_color_idx(color_stack, idx),
            ColorPalette::PerCharacter {
                char_map: _,
                fallback_color: _,
            } => {
                todo!(
                    "impl ColorPalette::colorize_idx for ColorPalette::PerCharacter... I have no idea how I will do this"
                );
            }
            ColorPalette::Uniform(val) => Ok(val.clone()),
        }
    }
}

const MONOKAI_FG: [&str; 7] = [
    //"#272822", //background,
    "#f8f8f2", //foreground
    "#f92672", //Pink / Magenta (Keywords):
    "#a6e22e", //Green (Strings / Insertions):
    "#66d9ef", //Blue (Functions / Methods):
    "#fd971f", //Orange (Constants / Numbers / Parameters):
    "#ae81ff", //Purple (Types / Keywords):
    "#75715e", //Comment:
];

//const MONOKAI_BG: &str = "#272822";

const VERIDIS_FG: [&str; 5] = [
    "#440154", //dark indigo
    "#3B528B", "#21908C", "#5DC863", "#FDE725", //sunflower yellow
];

//const VERIDIS_BG: &str = "#440154";

//#[derive(Debug, Serialize, Deserialize)]
//pub struct CustomPallette {
//    char_map: ,
//}
