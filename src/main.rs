use grammers_client::{Config, InitParams};
use grammers_session::Session;
use clap::Parser;
use dirs::home_dir;

mod app;
mod tg;
mod args;
mod dialogs;

#[tokio::main]
async fn main() {
    let arg = args::Arguments::parse();
    let path = arg.session_path.clone().unwrap_or_else(||{
        let mut h = home_dir().unwrap();
        h.push(".config");
        h.push("teleconsole");
        h.push("session");
        h
    });
    println!("Arguments {:?}", arg);
    println!("Session path: {:?}", path);

    let config = Config {
        api_hash: arg.api_hash,
        api_id: arg.api_id,
        params: InitParams::default(),
        session: Session::load_file_or_create(path.clone()).unwrap(),
    };
    if let Ok(mut a) = app::App::new(config, path).await {
        a.run().await;
    } else {
        eprintln!("Can`t start app");
    }
}
