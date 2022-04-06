use std::{io::Stdout, collections::HashMap, any::Any};

use crossterm::event::Event;
use dptree::prelude::DependencyMap;
use tui::{Frame, backend::CrosstermBackend};
use grammers_client::types::Update;


pub struct ArgumenDrawer<'a> {
    frame: &'a mut Frame<'a, CrosstermBackend<Stdout>>,
    deps: &'a DependencyMap,
    inputs: &'a Vec<Event>,
    events: &'a Option<Update>,
}

pub type Drawer = fn(arguments: ArgumenDrawer);//TODO: Переделать в динамические аргументы ?????
//TIPS: Продумать как передовать всю необходимую информацию во внутрь систем по цепочке
//TIPS: Система должна иметь доступ к глобальным объектам.


//TIPS: Подсистема работает только во время своей функции run
pub type Resolver = fn();

pub struct System<State>{
    pub state: State,
    pub drawer: HashMap<State, Vec<Drawer>>,
    pub sub_system: HashMap<State, Box<dyn ExecSystem<dyn Any>>>,
    pub resolver: HashMap<State, Resolver>
}

pub trait ExecSystem<State> {
    fn add_drawer(&mut self, drawer: Drawer);
    fn set_resolver(&mut self, resolver: Resolver);
    fn set_subsystem(&mut self, state: State, system: dyn ExecSystem<dyn Any>);
    fn run(&mut self);
}
//TODO: Реализовать рекурсивную структуру ECS с возможностью поддипов