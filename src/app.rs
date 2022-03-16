use std::{io::{self, Stdout}, sync::{Mutex, Arc}, borrow::Borrow};

use crossterm::event::{read, Event};
use grammers_client::{Client, Config, Update};
use tokio::sync::{mpsc};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders}};

use crate::tg;

#[derive(Debug, Clone, Copy)]
pub enum AppState {
    Login = 0,
    Dialogs = 1,
    Settings,
}

pub struct App {
    state: AppState,
    client: Arc<Mutex<Client>>,
    chats: Vec<i128>,
    open_chat: Option<i128>,
    inputs: mpsc::UnboundedReceiver<Event>,
    updates: mpsc::UnboundedReceiver<Update>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self, tg::TgErrors> {
        let (txk, rxk) = mpsc::unbounded_channel();
        tokio::spawn(async {
            while let Ok(event) = read() {
                txk.send(event);
            }
        });
        let client = Arc::new(Mutex::new(Client::connect(config).await?));
        let (txu, rxu) = mpsc::unbounded_channel();
        tokio::spawn(async {
            let c = client.clone();
            loop{
                match c.lock(){
                    Ok(client)=>{
                        match client.borrow().next_update().await {
                            Ok(Some(u)) => {
                                if let Err(e) = txu.send(u) {
                                    eprintln!("{}", e);
                                    return;
                                }
                            }
                            Ok(_)=>{}
                            Err(_) => {},
                        }
                    },
                    Err(e)=>{
                        eprintln!("{}", e);
                                return;
                    }
                }
            }
        });

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        Ok(App {
            state: AppState::Login,
            client,
            chats: Vec::new(),
            open_chat: None,
            inputs: rxk,
            updates: rxu,
            terminal,
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.state {
                AppState::Login => {
                    self.login().await;
                }
                _ => {}
            }
        }
    }

    pub async fn login(&mut self) {
        loop {
            self.terminal.draw(|f| {
                let size = f.size();
                let block = Block::default().title("Phone input").borders(Borders::ALL);
                f.render_widget(block, size);
            });
        }
    }
}
