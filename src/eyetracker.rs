use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EyetrackerState {
    x_pos: f32,
    y_pos: f32,
}
