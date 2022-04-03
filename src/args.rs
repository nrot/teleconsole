use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author = "nrot", version = "0.1a")]
pub struct Arguments {
    #[clap(short, long, default_value="~/.config/teleconsole/session")]
    pub session_path: PathBuf,
    #[clap(long, default_value_t = 5578726)]
    pub api_id: i32,
    #[clap(long, default_value = "c1449971f7d76221c6092cadc3617915")]
    pub api_hash: String,
}
