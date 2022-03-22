use std::io::{self, Stdout};

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use grammers_client::{types::LoginToken, Client, Config, Update};
use tokio::sync::mpsc;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
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
pub enum DialogState{
    DialogChose,
    DialogView,
    DialogInput
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
                AppState::Dialogs=>{
                    self.dialogs().await;
                }
                AppState::Exit => {
                    break;
                }
                _ => {
                    break;
                }
            }
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

    pub fn draw_dialogs(&mut self){
        
    }

    pub async fn dialogs(&mut self){
        let mut dialog_state = DialogState::DialogChose;
        loop {

            match dialog_state{
                DialogState::DialogChose=>{

                },
                DialogState::DialogView=>{},
                DialogState::DialogInput=>{},
                #[allow(unreachable_patterns)]
                _=>{break}
            }
        }
    }

    pub async fn login(&mut self) {
        let mut login_state = LoginState::PhoneInput;
        let mut login_token = None; //TIPS: This is just example
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
                            login_token = Some(s); //TIPS: Get LoginToken this
                            login_state = LoginState::CodeInput;
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
                        Ok(_) => {
                            self.state = AppState::Dialogs;
                            break;
                        }
                        Err(e) => {
                            self.draw_error(&e).await;
                            login_state = LoginState::PhoneInput;
                            login_token = None
                        }
                    }
                },
                #[allow(unreachable_patterns)]
                _=>{break}
            }
        }
    }
}
