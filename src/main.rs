use std::{
    fs::{File, OpenOptions},
    io::Read,
};

use clap::Parser;
use colcat::{
    renderer::Renderer,
    types::params::{ColcatParams, ColcatParamsSerde, ColcatTuiArgs},
};
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    if args.config.is_none() {
        println!("No config file provided, using default values!");
    }
    let config = args
        .config
        .map(ColcatParamsSerde::try_from)
        .unwrap_or(Ok(ColcatParamsSerde::default()))?;
    let params = ColcatParams::try_from(config)?;
    let mut file = File::open(args.input_file)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let text = String::from_utf8(buf)?;
    let renderer = Renderer::new(text, params)?;
    renderer.run()?;
    Ok(())
}
