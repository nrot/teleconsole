use std::{
    any::Any,
    collections::HashMap,
    io::{self, Stdout},
    path::PathBuf,
};

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use grammers_client::types::Update;
use grammers_client::{client::chats::InvocationError, Client, Config};
use tokio::sync::mpsc;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};

use crate::{
    dialogs::{DialogsSelected, OrderedDialogs},
    tg,
};

use dptree::di::DependencyMap;

#[derive(Debug, Clone, Copy)]
pub enum AppState {
    Prepare = 0,
    Login = 1,
    Dialogs = 2,
    Input,
    Settings,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub enum DialogState {
    DialogChose,
    DialogView,
    DialogInput,
}

#[derive(Debug, Clone, Copy)]
pub enum LoginState {
    PhoneInput,
    TokenRequest,
    CodeInput,
    CodeCheck,
}
pub struct App {
    state: AppState,
    dependecys: DependencyMap,
    drawers: HashMap<AppState, Vec<Drawer>>,
    client: Client,
    inputs: mpsc::UnboundedReceiver<Event>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    user_input_buf: String,
    chats: OrderedDialogs,
    session_path: PathBuf,
    api_id: i32,
    api_hash: String,
}

fn center(window: Rect, w: u16, h: u16) -> Rect {
    Rect {
        width: w,
        height: h,
        x: window.width / 2 - w / 2,
        y: window.height / 2 - h / 2,
    }
}

impl App {
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
        let deps = DependencyMap::new(); //.insert(item);
        Ok(App {
            state: AppState::Prepare,
            dependecys: deps,
            drawers: HashMap::new(),
            client: Client::connect(config).await?,
            inputs: rxk,
            terminal,
            user_input_buf: String::new(),
            chats: OrderedDialogs::new(),
            session_path: spath,
            api_id,
            api_hash,
        })
    }

    pub fn add_deps<T: Send + Sync + 'static>(self, item: T) -> Self {
        self
    }

    pub fn add_handler(self, state: AppState, handler: Drawer) -> Self {
        self
    }

    pub async fn run(&mut self) {
        enable_raw_mode().unwrap();
        self.terminal.clear().unwrap();
        if self.client.is_authorized().await.unwrap() {
            self.state = AppState::Dialogs;
        }
        loop {
            match self.state {
                AppState::Login => {
                    self.login().await;
                }
                AppState::Dialogs => {
                    self.dialogs().await;
                }
                AppState::Exit => {
                    break;
                }
                _ => {
                    break;
                }
            }
            self.user_input_buf = String::new();
            self.terminal.clear().unwrap();
        }
        self.inputs.close();
        disable_raw_mode().unwrap();
    }

    pub async fn draw_error<T: ToString>(&mut self, message: &T) {
        self.terminal
            .draw(|f| {
                let m = message.to_string();
                let inp = Paragraph::new(m.as_str())
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .title("Error")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Red)),
                    );
                f.render_widget(inp, center(f.size(), 20, 3));
            })
            .unwrap();
        self.inputs.recv().await;
    }

    pub fn draw_message(&mut self, title: &str, text: &str, size: Rect) {
        self.terminal
            .draw(|f| {
                let inp = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .block(Block::default().title(title).borders(Borders::ALL));
                f.render_widget(inp, center(f.size(), size.width, size.height));
            })
            .unwrap();
    }

    pub async fn wait_flud(&mut self, time: u32) {
        let mut now = chrono::Local::now();
        let end = chrono::Local::now() + chrono::Duration::seconds(time as i64);
        while now < end {
            now = chrono::Local::now();
            self.draw_message(
                "Wait flood",
                format!("End time: {}", end.timestamp() - now.timestamp()).as_str(),
                Rect {
                    width: 30,
                    height: 3,
                    ..Default::default()
                },
            );
        }
    }

    pub async fn draw_dialogs(&mut self) {
        let mut dialogs = self.client.iter_dialogs();
        self.chats.clear();
        let mut get_dialogs = true;
        while get_dialogs {
            match dialogs.next().await {
                Ok(Some(d)) => {
                    self.chats.insert(d);
                }
                Ok(None) => {
                    get_dialogs = false;
                }
                Err(e) => match e {
                    InvocationError::Rpc(r) => match r.name.as_str() {
                        "FLOOD_WAIT" => {
                            self.wait_flud(r.value.unwrap_or(1)).await;
                        }
                        e => {
                            self.draw_error(&e).await;
                        }
                    },
                    e => {
                        self.draw_error(&e).await;
                    }
                },
            }
        }
        self.terminal
            .draw(|f| {
                let d = self.chats.clone();
                let mut st = DialogsSelected { selected: 1 };
                f.render_stateful_widget(d, f.size(), &mut st);
            })
            .unwrap();
        self.inputs.recv().await;
    }

    pub async fn dialogs(&mut self) {
        let mut dialog_state = DialogState::DialogChose;
        loop {
            self.draw_dialogs().await;
            match dialog_state {
                DialogState::DialogChose => {}
                DialogState::DialogView => {}
                DialogState::DialogInput => {}
                #[allow(unreachable_patterns)]
                _ => break,
            }
        }
    }

    pub async fn load_data(&mut self) {
        self.client
            .session()
            .save_to_file(self.session_path.clone())
            .unwrap();
    }

    pub async fn login(&mut self) {
        let mut login_state = LoginState::PhoneInput;
        let mut login_token = None;
        loop {
            match login_state {
                LoginState::PhoneInput => {
                    let buf = self.user_input_buf.clone();
                    self.draw_message(
                        "Phone input",
                        buf.as_str(),
                        Rect {
                            width: 20,
                            height: 3,
                            ..Default::default()
                        },
                    );
                    while let Ok(e) = self.inputs.try_recv() {
                        #[allow(clippy::single_match)]
                        match e {
                            Event::Key(c) => match c.code {
                                KeyCode::Backspace => {
                                    self.user_input_buf.pop();
                                }
                                KeyCode::Char(c) => {
                                    if c.is_ascii_digit() || c == '+' {
                                        self.user_input_buf.push(c)
                                    }
                                }
                                KeyCode::Esc => {
                                    self.state = AppState::Exit;
                                    return;
                                }
                                KeyCode::Enter => login_state = LoginState::TokenRequest,
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
                LoginState::TokenRequest => {
                    match self
                        .client
                        .request_login_code(
                            self.user_input_buf.as_str(),
                            self.api_id,
                            self.api_hash.as_str(),
                        )
                        .await
                    {
                        Ok(s) => {
                            login_token = Some(s);
                            login_state = LoginState::CodeInput;
                            self.user_input_buf = String::new();
                        }
                        Err(e) => {
                            self.draw_error(&e).await;
                            login_state = LoginState::PhoneInput;
                        }
                    };
                }
                LoginState::CodeInput => {
                    let buf = self.user_input_buf.clone();
                    self.draw_message(
                        "Code input",
                        buf.as_str(),
                        Rect {
                            width: 20,
                            height: 3,
                            ..Default::default()
                        },
                    );
                    while let Ok(e) = self.inputs.try_recv() {
                        #[allow(clippy::single_match)]
                        match e {
                            Event::Key(c) => match c.code {
                                KeyCode::Backspace => {
                                    self.user_input_buf.pop();
                                }
                                KeyCode::Char(c) => {
                                    if c.is_ascii_digit() {
                                        self.user_input_buf.push(c)
                                    }
                                }
                                KeyCode::Esc => {
                                    self.state = AppState::Exit;
                                    return;
                                }
                                KeyCode::Enter => login_state = LoginState::CodeCheck,
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
                LoginState::CodeCheck => {
                    match self
                        .client
                        .sign_in(login_token.as_ref().unwrap(), self.user_input_buf.as_str())
                        .await
                    {
                        Ok(s) => {
                            self.load_data().await;
                            self.state = AppState::Dialogs;
                            break;
                        }
                        Err(e) => {
                            self.draw_error(&e).await;
                            login_state = LoginState::PhoneInput;
                            login_token = None
                        }
                    }
                }
                #[allow(unreachable_patterns)]
                _ => break,
            }
        }
    }
}
