use std::{collections::HashMap, fmt::Debug, fs::read_to_string, ops::Range, path::PathBuf};

use candle_nn::VarBuilder;
use candle_transformers::models::{
    bert::{BertModel, DTYPE},
    mimi::candle::{Device, Tensor},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokenizers::{Encoding, PaddingParams, Tokenizer};
use tracing::debug;

use crate::{
    color::words::LinguisticColoring,
    errors::{ColcatError, ColcatSemanticsError},
};

pub struct SemanticsModel {
    model: BertModel,
    tokenizer: Tokenizer,
    config: candle_transformers::models::bert::Config,
    target_text: Vec<String>,
    device: Device,
    //model_path: PathBuf,
    //tokenizer_path: PathBuf,
    //config_path: PathBuf,
}

impl Debug for SemanticsModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "model: BertModel (data hidden), tokenizer: {:?}, config: {:?}",
            self.tokenizer, self.config
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SemanticsParamsSerde {
    model_path: PathBuf,
    tokenizer_path: PathBuf,
    config_path: PathBuf,
    target_text: Vec<String>,
    //approximate_gelu: Option<bool>?
}
//impl Default for SemanticsModelSerde {
//
//}

//#[derive(Debug, Deserialize)]
//pub struct ConfigWrapper(candle_transformers::models::bert::Config);

impl SemanticsModel {
    pub fn new(serde_data: SemanticsParamsSerde) -> Result<Self, ColcatError> {
        if serde_data.target_text.is_empty() {
            return Err(ColcatSemanticsError::TargetTextCannotBeEmpty.into());
        }
        let device = Device::Cpu;
        let config_string = read_to_string(serde_data.config_path)?;
        let mut config = serde_json::from_str(&config_string)?;
        let mut tokenizer =
            Tokenizer::from_file(serde_data.tokenizer_path).map_err(ColcatSemanticsError::from)?;
        let vb = VarBuilder::from_pth(serde_data.model_path, DTYPE, &device)
            .map_err(ColcatSemanticsError::from)?;
        let model = BertModel::load(vb, &config).map_err(ColcatSemanticsError::from)?;
        let target_text = serde_data.target_text.clone();

        //save on performance?
        config.hidden_act = candle_transformers::models::bert::HiddenAct::GeluApproximate;

        if let Some(padding_params) = tokenizer.get_padding_mut() {
            padding_params.strategy = tokenizers::PaddingStrategy::BatchLongest
        } else {
            let padding_params = PaddingParams {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            };
            tokenizer.with_padding(Some(padding_params));
        }

        Ok(Self {
            model,
            //model_path,
            tokenizer,
            //tokenizer_path,
            config,
            //config_path,
            target_text,
            device,
        })
    }

    pub fn infer(
        &self,
        text: &str,
        split_mode: &LinguisticColoring,
    ) -> Result<Vec<ProcessedSemantics>, ColcatSemanticsError> {
        let mut split_vals = match split_mode {
            LinguisticColoring::Words => text
                .split_whitespace()
                .map(str::trim)
                .map(|val| val.replace("\n", ""))
                .filter(|val| !val.is_empty())
                //.map(str::to_string)
                .collect_vec(),
            LinguisticColoring::Sentences { delimiter } => text
                .split(*delimiter)
                .map(str::trim)
                .map(|val| val.replace("\n", ""))
                .filter(|val| !val.is_empty())
                //.map(str::to_string)
                .collect_vec(),
            //why would you do this? Whatever, it's possible
            LinguisticColoring::Characters => text.chars().map(String::from).collect_vec(),
        };
        let input_num_sentences = split_vals.len();
        debug!("input_num_sentences: {input_num_sentences}");
        let mut target_text = self.target_text.clean_text();
        debug!("target_text: {target_text:?}");
        split_vals.append(&mut target_text);
        let total_num_sentences = split_vals.len();
        let input = self
            .tokenizer
            .encode_batch(split_vals.clone(), true)
            .map_err(ColcatSemanticsError::TokenizerEncodeBatchFail)?;
        debug!("tokens: {input:?}");
        let token_ids = input.get_token_ids(&self.device)?;
        debug!("token_ids: {token_ids:?}");

        let attention_mask = input
            .iter()
            .map(|token| {
                let tokens = token.get_attention_mask();
                Ok(Tensor::new(tokens, &self.device)?)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(ColcatSemanticsError::TensorAllocationFail)?;
        debug!("attention_mask: {attention_mask:?}");

        let token_id_stack = Tensor::stack(&token_ids, 0)?;

        let attention_mask = Tensor::stack(&attention_mask, 0)?;

        let token_type_ids = token_id_stack.zeros_like()?;
        debug!("Running inferences on batch {:?}", token_id_stack.shape());
        let embeddings =
            self.model
                .forward(&token_id_stack, &token_type_ids, Some(&attention_mask))?;
        debug!("generated embeddings {:?}", embeddings.shape());

        let args_include_padding_embeddings = true;
        //pooling
        let embeddings = if args_include_padding_embeddings {
            let (num_sentences, num_tokens, hidden_size) = embeddings.dims3()?;
            (embeddings.sum(1)? / (num_tokens as f64))?
        } else {
            //sentence transformers behavior
            let attention_mask_for_pooling = attention_mask.to_dtype(DTYPE)?.unsqueeze(2)?;
            let sum_mask = attention_mask_for_pooling.sum(1)?;
            let embeddings = (embeddings.broadcast_mul(&attention_mask_for_pooling)?).sum(1)?;
            embeddings.broadcast_div(&sum_mask)?
        };

        let args_normalize_embeddings = false;
        let embeddings = if args_normalize_embeddings {
            normalize_l2(&embeddings)?
        } else {
            embeddings
        };
        debug!("pooled embeddings: {:?}", embeddings.shape());

        let input_range = 0..input_num_sentences;
        let target_text_range = input_num_sentences..total_num_sentences;
        debug!("input_range: {input_range:?}, target_text_range: {target_text_range:?}");

        let processed =
            ProcessedSemantics::create(embeddings, split_vals, input_range, target_text_range)?;

        Ok(processed)
    }
}

//TODO: convert into a useful singleton object holding an array
//of processed phrases
#[derive(Debug)]
pub struct ProcessedSemantics {
    pub phrase: String,
    pub phrase_score_map: HashMap<String, f32>,
    pub phrase_embeddings: HashMap<String, Tensor>,
}

impl ProcessedSemantics {
    ///Make it so that you have a contiguous stack of tensors with input after the target text
    pub fn create(
        embeddings: Tensor,
        total_text: Vec<String>,
        input_text_idxs: Range<usize>,
        target_text_idxs: Range<usize>,
    ) -> Result<Vec<Self>, ColcatSemanticsError> {
        let mut ret_phrases = Vec::new();
        for j in target_text_idxs {
            let mut phrase_score_map = HashMap::new();
            let mut phrase_embeddings = HashMap::new();
            let embedding_j = embeddings.get(j)?;
            for i in input_text_idxs.clone() {
                let embedding_i = embeddings.get(i)?;
                //debug!("embedding(i={i},j={j})");
                let sum_ij = (&embedding_i * &embedding_j)?
                    .sum_all()?
                    .to_scalar::<f32>()?;
                let sum_i2 = (&embedding_i * &embedding_i)?
                    .sum_all()?
                    .to_scalar::<f32>()?;
                let sum_j2 = (&embedding_j * &embedding_j)?
                    .sum_all()?
                    .to_scalar::<f32>()?;
                let cosine_similarity = sum_ij / (sum_i2 * sum_j2).sqrt();
                let text = total_text[i].clone();
                phrase_score_map.insert(text.clone(), cosine_similarity);
                phrase_embeddings.insert(text, embedding_i);
            }
            let phrase = total_text[j].clone();
            ret_phrases.push(Self {
                phrase,
                phrase_score_map,
                phrase_embeddings,
            });
        }
        Ok(ret_phrases)
    }
}

//similarities.sort_by(|u, v| v.0.total_cmp(&u.0));
//for &(score, i, j) in similarities[..5].iter() {
//    debug!(
//        "score: {score:.2}, '{}' '{}'",
//        &split_vals[i], &split_vals[j]
//    );
//}

fn normalize_l2(v: &Tensor) -> Result<Tensor, ColcatSemanticsError> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}

trait TokenProcess {
    fn get_token_ids(&self, device: &Device) -> Result<Vec<Tensor>, ColcatSemanticsError>;
}

impl TokenProcess for Vec<Encoding> {
    fn get_token_ids(&self, device: &Device) -> Result<Vec<Tensor>, ColcatSemanticsError> {
        self.iter()
            .map(|token| {
                let tokens = token.get_ids();
                Ok(Tensor::new(tokens, device)?)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(ColcatSemanticsError::TensorAllocationFail)
    }
}

pub trait CleanupText {
    fn clean_text(&self) -> Self;
}

impl CleanupText for Vec<String> {
    fn clean_text(&self) -> Self {
        self.iter()
            .filter(|val| !val.is_empty())
            .map(|val| val.trim())
            .map(|val| val.replace("\n", ""))
            .collect_vec()
    }
}
