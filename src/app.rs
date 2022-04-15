use std::any::Any;
use std::collections::HashMap;
use std::io;
use std::{path::PathBuf, io::Stdout};

use crossterm::event::Event;
use crossterm::event::read;
use crossterm::terminal::enable_raw_mode;
use grammers_client::{Client, Config};
use tokio::sync::mpsc;
use tui::{Terminal, backend::CrosstermBackend};

use crate::{ecs, tg};

pub struct App{
    client: Client,
    inputs: mpsc::UnboundedReceiver<Event>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    session_path: PathBuf,
    api_id: i32,
    api_hash: String,
    systems: ecs::SystemList
}

impl App{
    pub async fn new(config: Config, spath: PathBuf) -> Result<Self, tg::TgErrors> {
        let (txk, rxk) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Ok(event) = read() {
                txk.send(event).unwrap();
            }
        });

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        let api_id = config.api_id;
        let api_hash = config.api_hash.clone();
        Ok(App {
            client: Client::connect(config).await?,
            inputs: rxk,
            systems: HashMap::new(),
            terminal,
            session_path: spath,
            api_id,
            api_hash,
        })
    }

    pub async fn run(&mut self){
        enable_raw_mode().unwrap();
        self.terminal.clear().unwrap();

    }
}