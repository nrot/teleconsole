use grammers_client::{Config, InitParams};
use grammers_session::Session;
use clap::Parser;

mod app;
mod tg;
mod args;
mod dialogs;

#[tokio::main]
async fn main() {
    let arg = args::Arguments::parse();
    let path = arg.session_path.canonicalize().unwrap();
    println!("Arguments {:?}", arg);

    let config = Config {
        api_hash: arg.api_hash,
        api_id: arg.api_id,
        params: InitParams::default(),
        session: Session::load_file_or_create(path).unwrap(),
    };
    if let Ok(mut a) = app::App::new(config).await {
        a.run().await;
    } else {
        eprintln!("Can`t start app");
    }
}
