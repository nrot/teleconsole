use std::io::{self, Stdout};

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use grammers_client::{types::LoginToken, Client, Config, Update};
use tokio::sync::mpsc;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::tg;

#[derive(Debug, Clone, Copy)]
pub enum AppState {
    Login = 0,
    Dialogs = 1,
    Input,
    Settings,
    Exit,
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
    client: Client,
    chats: Vec<i128>,
    open_chat: Option<i128>,
    inputs: mpsc::UnboundedReceiver<Event>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    user_input_buf: String,
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
    pub async fn new(config: Config) -> Result<Self, tg::TgErrors> {
        let (txk, rxk) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Ok(event) = read() {
                txk.send(event).unwrap();
            }
        });

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        let api_id = config.api_id.clone();
        let api_hash = config.api_hash.clone();
        Ok(App {
            state: AppState::Login,
            client: Client::connect(config).await?,
            chats: Vec::new(),
            open_chat: None,
            inputs: rxk,
            terminal,
            user_input_buf: String::new(),
            api_id,
            api_hash,
        })
    }

    pub async fn run(&mut self) {
        enable_raw_mode().unwrap();
        self.terminal.clear().unwrap();
        loop {
            match self.state {
                AppState::Login => {
                    self.login().await;
                }
                AppState::Exit => {
                    break;
                }
                _ => {}
            }
        }
        self.inputs.close();
        disable_raw_mode().unwrap();
    }

    pub async fn login(&mut self) {
        let mut login_state = LoginState::PhoneInput;
        let mut login_token = None;
        loop {
            match login_state {
                LoginState::PhoneInput => {
                    let buf = self.user_input_buf.clone();
                    self.terminal
                        .draw(|f| {
                            let inp = Paragraph::new(buf.as_ref())
                                .alignment(tui::layout::Alignment::Center)
                                .block(Block::default().title("Phone input").borders(Borders::ALL));
                            f.render_widget(inp, center(f.size(), 20, 3));
                        })
                        .unwrap();
                    while let Ok(e) = self.inputs.try_recv() {
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
                        }
                        Err(e) => {
                            let err = e.to_string();
                            self.terminal
                                .draw(|f| {
                                    let inp = Paragraph::new(err.as_str())
                                        .alignment(tui::layout::Alignment::Center)
                                        .block(
                                            Block::default()
                                                .title("Auth error")
                                                .borders(Borders::ALL),
                                        );
                                    f.render_widget(inp, center(f.size(), 20, 3));
                                })
                                .unwrap();
                            let _ = self.inputs.recv().await;
                            login_state = LoginState::PhoneInput;
                        }
                    };
                }
                LoginState::CodeInput => {
                    let buf = self.user_input_buf.clone();
                    self.terminal
                        .draw(|f| {
                            let inp = Paragraph::new(buf.as_ref())
                                .alignment(tui::layout::Alignment::Center)
                                .block(Block::default().title("Code input").borders(Borders::ALL));
                            f.render_widget(inp, center(f.size(), 20, 3));
                        })
                        .unwrap();
                    while let Ok(e) = self.inputs.try_recv() {
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
                    let Some(lg) = login_token;
                    match self.client.sign_in(&lg, self.user_input_buf.as_str()).await {
                        Ok(s) => {}
                        Err(e) => {}
                    }
                }
            }
        }
    }
}
