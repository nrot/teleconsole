use std::collections::HashMap;
use std::io;
use std::rc::Rc;
use std::{path::PathBuf, io::Stdout};

use crossterm::event::Event;
use crossterm::event::read;
use crossterm::terminal::enable_raw_mode;
use dptree::prelude::DependencyMap;
use futures::future::select;
use grammers_client::{Client, Config};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tui::{Terminal, backend::CrosstermBackend};

use crate::ecs::{SystemState, step};
use crate::{ecs, tg};

pub struct App{
    client: Client,
    inputs: mpsc::UnboundedReceiver<Event>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    session_path: PathBuf,
    api_id: i32,
    api_hash: String,
    systems: ecs::SystemList,
    global: Rc<Mutex<DependencyMap>>,
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
        let global = Rc::new(Mutex::new(DependencyMap::new()));
        Ok(App {
            client: Client::connect(config).await?,
            inputs: rxk,
            systems: HashMap::new(),
            terminal,
            session_path: spath,
            api_id,
            api_hash,
            global
        })
    }

    pub fn add_system(&mut self, mut system: ecs::System<SystemState>){
        let s = system.id();
        system.global = self.global.clone();
        self.systems.insert(s, system);
    }

    pub async fn run(&mut self){
        enable_raw_mode().unwrap();
        self.terminal.clear().unwrap();
        let it = Box::pin(self.inputs.recv());
        let ut = Box::pin(self.client.next_update());
        let sel = select(it, ut);
        let w = match sel.await{
            futures::future::Either::Left((it, _)) => {
                step(&mut self.terminal, &mut self.systems, it, None).await
            },
            futures::future::Either::Right((ut, _)) => {
                match ut{
                    Ok(ut)=>{
                        step(&mut self.terminal, &mut self.systems, None, ut).await
                    },
                    Err(_)=>{
                        println!("");
                        false
                    }
                }
                
            },
        };
        // ecs::step(&mut self.terminal, &mut self.systems, input, update);
    }
    pub fn get_global(&self)->Rc<Mutex<DependencyMap>>{
        self.global.clone()
    }
}