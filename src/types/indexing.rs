//use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::ColcatError;

///How should colors translate to numbered elements?
///Translates into a series of numbers between 0.0 and 1.0
///where 0.0 is the beginning of the color palette and 1.0
///is the end
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum IndexingScale {
    ///the number that represents the char is the index for coloring
    #[default]
    Linear,
    //The common letters will be similar characters, while rare characters will be a different set
    //of colors
    //LetterFrequencyLinear,
    //LetterFrequencyGeometric, //like Zeno's paradox
    //Custom(HashMap<usize, f64>),
}

impl IndexingScale {
    pub fn scale(&self, idx: f64, min: f64, max: f64) -> Result<f64, ColcatError> {
        let idx = match self {
            IndexingScale::Linear => Ok(idx),
            //IndexingScale::Custom(map) => map
            //    .get(&(character as usize))
            //    .copied()
            //    .ok_or(ColcatError::UnmappedCustomIndex(character as usize))?,
        };
        idx
    }
}
