use chrono::prelude::Utc;
use grammers_client::types::Dialog;
use tui::{widgets::{List, ListItem, StatefulWidget, Borders, Block, ListState}, style::{Style, Color}};
use grammers_tl_types as tl;

#[derive(Debug, Clone)]
pub struct DialogsSelected {
    pub selected: i64,
}

#[derive(Debug, Clone)]
pub struct OrderedDialogs {
    header: Vec<i64>,
    hidden: Vec<i64>,
    all: Vec<Dialog>,
}

impl OrderedDialogs {
    pub fn new() -> Self {
        OrderedDialogs {
            header: Vec::new(),
            all: Vec::new(),
            hidden: Vec::new(),
        }
    }

    fn width(&self, d: &Dialog) -> i64 {
        let t = Utc::now();
        let mut w: i64 = if !self.header.contains(&d.chat.id()) {
            u8::MAX as i64
        } else {
            0
        };
        if let Some(m) = &d.last_message {
            w += t.timestamp() - m.date().naive_utc().timestamp();
        }
        w
    }

    pub fn insert(&mut self, d: Dialog) {
        let w = self.width(&d);
        let k = self.all.partition_point(|a| self.width(a) < w);
        self.all.insert(k, d);
    }

    pub fn clear(&mut self) {
        self.all.clear();
    }

    pub fn list(&self) -> Vec<Dialog> {
        self.all
            .iter()
            .filter_map(|d| {
                if !self.hidden.contains(&d.chat.id()) {
                    Some(d.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[inline]
fn display_count(cnt: String)->String{
    if cnt.is_empty(){
        String::new()
    } else {
        format!("({})", cnt)
    }
}

fn display_name(name: &str, width: usize, ucnt: i32)->String{
    let cnt = if ucnt > 0 {
        format!("{}", ucnt)
    } else {
        String::new()
    };
    if name.len() > width - cnt.len(){
        format!("{}..{}", name.chars().take(width-cnt.len()).fold(String::new(), |a, b|{
            a + b.to_string().as_str()
        }), display_count(cnt))
    } else {
        format!("{}{}", name, display_count(cnt))
    }
}

impl StatefulWidget for OrderedDialogs {
    type State = DialogsSelected;
    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let mut items = Vec::new();
        let name_size = (area.width - 1) as usize;
        let count = (area.height / 2) as i128;
        let dialogs = self.list();
        let index = dialogs
            .iter()
            .position(|d| d.chat.id() == state.selected)
            .unwrap_or(0);
        for (i, dialog) in dialogs.iter().enumerate() {
            if (i as i128) > (index as i128) - count && (i as i128) < (index as i128) + count {
                let s = match &dialog.dialog{
                    tl::enums::Dialog::Dialog(d)=>{
                        format!("D: {}", display_name(dialog.chat.name(), name_size - 3, d.unread_count)) 
                    },
                    tl::enums::Dialog::Folder(f)=>{
                        format!("F: {}", display_name(dialog.chat.name(), name_size - 3, f.unread_unmuted_messages_count)) 
                    }
                };
                items.push(if i == index{
                    ListItem::new(s).style(Style::default().bg(Color::LightGreen))
                } else {
                    ListItem::new(s)
                })
            }
        }
        let mut slct = ListState::default();
        slct.select(Some(index));
        let lst = List::new(items).block(Block::default().borders(Borders::ALL));
        lst.render(area, buf, &mut slct);
    }
}
