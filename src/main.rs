use grammers_client::{Config, InitParams};
use grammers_session::Session;

mod app;
mod tg;
// mod tui;

const API_ID: i32 = 5578726;
const API_HASH: &str = "c1449971f7d76221c6092cadc3617915";

#[tokio::main]
async fn main() {
    let config = Config {
        api_hash: String::from(API_HASH),
        api_id: API_ID,
        params: InitParams::default(),
        session: Session::new(),
    };
    if let Ok(mut a) = app::App::new(config).await{
        a.run().await;
    }else {
        eprintln!("Can`t start app");
    }
}
