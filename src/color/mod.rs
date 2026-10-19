use std::ops::{Add, Div, Mul, Sub};

use itertools::Itertools;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use vecstat::calc_traits::{Floatable, Zeroable};

pub mod palettes;
pub mod params;
#[cfg(feature = "semantics")]
pub mod semantics;
pub mod words;

//pub type ColorColcat = csscolorparser::Color;
pub type ColorColcat = ColorWrap;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Default, Clone)]
pub struct ColorWrap(csscolorparser::Color);

impl JsonSchema for ColorWrap {
    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::ColorWrap").into()
    }
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ColorWrap".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let _ = generator;
        json_schema!({
            "type": "css_color"
        })
    }
}

impl Add for ColorWrap {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let color_one = self.into_inner();
        let color_two = rhs.into_inner();
        let r = color_one.r + color_two.r;
        let g = color_one.g + color_two.g;
        let b = color_one.b + color_two.b;
        let a = color_one.a + color_two.a;
        Self(csscolorparser::Color::new(r, g, b, a))
    }
}

impl Sub for ColorWrap {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let color_one = self.into_inner();
        let color_two = rhs.into_inner();
        let r = color_one.r - color_two.r;
        let g = color_one.g - color_two.g;
        let b = color_one.b - color_two.b;
        let a = color_one.a - color_two.a;
        Self(csscolorparser::Color::new(r, g, b, a))
    }
}

impl Mul for ColorWrap {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let color_one = self.into_inner();
        let color_two = rhs.into_inner();
        let r = color_one.r * color_two.r;
        let g = color_one.g * color_two.g;
        let b = color_one.b * color_two.b;
        let a = color_one.a * color_two.a;
        Self(csscolorparser::Color::new(r, g, b, a))
    }
}

impl Div<f32> for ColorWrap {
    type Output = ColorWrap;
    fn div(self, rhs: f32) -> Self::Output {
        let color_one = self.into_inner();
        let r = color_one.r / rhs;
        let g = color_one.g / rhs;
        let b = color_one.b / rhs;
        let a = color_one.a / rhs;
        Self(csscolorparser::Color::new(r, g, b, a))
    }
}

impl Zeroable for ColorWrap {
    fn zero() -> Self {
        Self(csscolorparser::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        })
    }
}

impl Floatable<f32> for ColorWrap {
    fn powi(self, n: i32) -> Self {
        let color_one = self.into_inner();
        let r = color_one.r.powi(n);
        let g = color_one.g.powi(n);
        let b = color_one.b.powi(n);
        let a = color_one.a.powi(n);
        Self(csscolorparser::Color::new(r, g, b, a))
    }
    fn powf(self, n: f32) -> Self {
        let color_one = self.into_inner();
        let r = color_one.r.powf(n);
        let g = color_one.g.powf(n);
        let b = color_one.b.powf(n);
        let a = color_one.a.powf(n);
        Self(csscolorparser::Color::new(r, g, b, a))
    }
    fn sqrt(self) -> Self {
        let color_one = self.into_inner();
        let r = color_one.r.sqrt();
        let g = color_one.g.sqrt();
        let b = color_one.b.sqrt();
        let a = color_one.a.sqrt();
        Self(csscolorparser::Color::new(r, g, b, a))
    }
}

impl From<csscolorparser::Color> for ColorWrap {
    fn from(value: csscolorparser::Color) -> Self {
        Self(value)
    }
}

impl ColorWrap {
    pub fn new(color: csscolorparser::Color) -> Self {
        Self(color)
    }
    pub fn inner(&self) -> &csscolorparser::Color {
        &self.0
    }
    pub fn inner_mut(&mut self) -> &mut csscolorparser::Color {
        &mut self.0
    }
    pub fn into_inner(self) -> csscolorparser::Color {
        self.0
    }
    pub fn from_hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self(csscolorparser::Color::from_hsva(h, s, v, a))
    }
    pub fn interpolate_rgb(&self, other: &Self, distance: f32) -> Self {
        self.inner().interpolate_rgb(other.inner(), distance).into()
    }
    pub fn black() -> Self {
        Self::zero()
    }
}

impl Eq for ColorWrap {}
//impl Ord for ColorWrap {
//    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//        match self.inner().partial_cmp(other.inner()) {
//            Some(cmp) => cmp,
//            None => panic!("Failed to cmp self vs other"),
//        }
//    }
//}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColoredCharacter {
    pub fg_color: ColorColcat,
    pub bg_color: Option<ColorColcat>,
    pub character: char,
}

//#[derive(Debug, Serialize, Deserialize, Clone)]
//pub struct CharacterColoring {
//    pub fg_color: ColorColcat,
//    pub bg_color: Option<ColorColcat>,
//}

pub trait SplitColoredCharacterLines {
    fn split_char(self, delimiter: char) -> Vec<Self>
    where
        Self: Sized;
}

impl SplitColoredCharacterLines for Vec<ColoredCharacter> {
    fn split_char(self, delimiter: char) -> Vec<Self> {
        self.split(|val| val.character == delimiter)
            .map(|val| val.to_vec())
            .collect_vec()
    }
}

pub trait JoinColoredCharacterLines {
    type Output;
    fn join_char(self, delimiter: char) -> Self::Output;
}

impl JoinColoredCharacterLines for Vec<Vec<ColoredCharacter>> {
    type Output = Vec<ColoredCharacter>;
    fn join_char(self, delimiter: char) -> Self::Output {
        self.join(&ColoredCharacter {
            character: delimiter,
            fg_color: ColorColcat::zero(),
            bg_color: None,
        })
    }
}

impl ColoredCharacter {
    pub fn new(fg_color: ColorColcat, bg_color: Option<ColorColcat>, character: char) -> Self {
        Self {
            fg_color,
            bg_color,
            character,
        }
    }
    pub fn abs(self) -> Self {
        let fg_color = ColorColcat::new(csscolorparser::Color {
            r: self.fg_color.inner().r.abs(),
            g: self.fg_color.inner().g.abs(),
            b: self.fg_color.inner().b.abs(),
            a: self.fg_color.inner().a.abs(),
        });
        let bg_color = self.bg_color.map(|bg_color| {
            ColorColcat::new(csscolorparser::Color {
                r: bg_color.inner().r.abs(),
                g: bg_color.inner().g.abs(),
                b: bg_color.inner().b.abs(),
                a: bg_color.inner().a.abs(),
            })
        });
        Self {
            fg_color,
            bg_color,
            character: self.character,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub enum ColoringScheme {
    ForegroundOnly,
    BackgroundOnly,
    #[default]
    ForegroundAndBackground,
}
