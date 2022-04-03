use std::cmp::Ordering;

use chrono::prelude::Utc;
use grammers_client::types::Dialog;

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

    pub fn list(&mut self) -> Vec<Dialog> {
        self.all.iter().filter_map(|d|{
            if !self.hidden.contains(&d.chat.id()) {
                Some(d.clone())
            } else {
                None
            }
        }).collect()
    }
}
