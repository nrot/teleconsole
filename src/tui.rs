use std::io;
use tui::{
    backend::TermionBackend,
    widgets::{Block, Borders},
    Terminal,
};


use super::{API_HASH, API_ID};
use crate::tg;

use grammers_client::InitParams;
use grammers_session::Session;


pub async fn window(){
    let stdout = io::stdout();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Phone input")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    });

    
}
