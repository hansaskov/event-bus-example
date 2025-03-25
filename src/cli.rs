use crate::modules::uploader;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(flatten)]
    pub upload_config: uploader::Config,
}
