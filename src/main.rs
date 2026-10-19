use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::absolute,
};

use clap::Parser;
use colcat::{
    errors::ColcatError,
    renderer::Renderer,
    types::params::{ColcatParams, ColcatParamsSerde, ColcatTuiArgs},
};
use schemars::schema_for;
use tracing::debug;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let debug_file = OpenOptions::new()
        //.append(true)
        .write(true)
        .append(false)
        .create(true)
        .truncate(true)
        .open("debug.log")?;
    let subscriber = tracing_subscriber::fmt()
        .with_writer(debug_file)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let args = ColcatTuiArgs::parse();
    if let Some(schema_path) = args.generate_config_schema {
        let schema = schema_for!(ColcatParamsSerde);
        let out_text = serde_json::to_string_pretty(&schema)?;
        let mut file = File::create(schema_path)?;
        file.write_all(out_text.as_bytes())?;
        return Ok(());
    }
    if args.input_file.is_none() {
        return Err(ColcatError::MissingInputFile.into());
    }
    if args.config.is_none() {
        debug!("No config file provided, using default values!");
        println!("No config file provided, using default values!");
    }
    let config = args
        .config
        .map(ColcatParamsSerde::try_from)
        .unwrap_or(Ok(ColcatParamsSerde::default()))?;
    let params = ColcatParams::try_from(config)?;
    let mut file = File::open(absolute(args.input_file.unwrap())?)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let text = String::from_utf8(buf)?;
    let renderer = Renderer::new(text, params)?;
    renderer.run()?;
    Ok(())
}
