use std::{any::Any, collections::HashMap, future::Future, io::Stdout, pin::Pin, rc::Rc};

use async_trait::async_trait;
use crossterm::event::Event;
use dptree::prelude::DependencyMap;
use grammers_client::types::Update;
use tokio::sync::Mutex;
use tui::{backend::CrosstermBackend, Frame};

pub struct ArgumenDrawer<'a> {
    frame: &'a mut Frame<'a, CrosstermBackend<Stdout>>,
    deps: &'a DependencyMap,
    inputs: &'a Vec<Event>,
    events: &'a Option<Update>,
}

pub type Drawer = fn(arguments: ArgumenDrawer) -> Pin<Box<dyn Future<Output = ()>>>; //TODO: Переделать в динамические аргументы ?????
                                                                                     //TIPS: Продумать как передовать всю необходимую информацию во внутрь систем по цепочке
                                                                                     //TIPS: Система должна иметь доступ к глобальным объектам.

//TIPS: Подсистема работает только во время своей функции run
pub type Resolver = fn();

pub struct System<State> {
    pub state: State,
    pub drawer: HashMap<State, Vec<Drawer>>,
    pub global: Rc<Mutex<DependencyMap>>,
    pub local: Rc<Mutex<DependencyMap>>,
    pub sub_system: HashMap<State, Box<dyn ExecSystem<dyn Any>>>,
    pub resolver: HashMap<State, Resolver>,
}

impl<State: Clone> System<State> {
    fn new(istate: State, global: Rc<Mutex<DependencyMap>>) -> Self {
        System {
            state: istate,
            drawer: HashMap::new(),
            global: global.clone(),
            local: Rc::new(Mutex::new(DependencyMap::new())),
            sub_system: HashMap::new(),
            resolver: HashMap::new(),
        }
    }
}

#[async_trait]
pub trait ExecSystem<State: Sized> {
    fn add_drawer(&mut self, drawer: Drawer);
    fn set_resolver(&mut self, resolver: Resolver);
    fn set_subsystem(&mut self, state: State, system: dyn ExecSystem<dyn Any>);
    async fn run(&mut self, f: &'static mut Frame<'static, CrosstermBackend<Stdout>>);
}
//TODO: Реализовать рекурсивную структуру ECS с возможностью подтипов


