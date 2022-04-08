use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    hash::Hash,
    io::Stdout,
    pin::Pin,
    rc::Rc,
    sync::Arc,
};

use async_trait::async_trait;
use crossterm::event::Event;
use dptree::di::{DependencyMap, DependencySupplier};
use futures::{future::select, select, Future};
use grammers_client::{types::Update, Client};
use tokio::sync::{mpsc, Mutex, MutexGuard};
use tui::{backend::CrosstermBackend, Frame, Terminal};

// use crate::di::{Dependency, DependencyMap};

// pub const GTERMINAL: &str = "TERMINAL";

pub struct ArgumenDrawer<'a> {
    frame: &'a mut Frame<'a, CrosstermBackend<Stdout>>,
    global: Rc<Mutex<DependencyMap>>,
    local: Rc<Mutex<DependencyMap>>,
    inputs: &'a Option<Event>,
    events: &'a Option<Update>,
}

pub type Drawer = fn(arguments: ArgumenDrawer) -> Pin<Box<dyn Future<Output = ()>>>; //TODO: Переделать в динамические аргументы ?????
                                                                                     //TIPS: Продумать как передовать всю необходимую информацию во внутрь систем по цепочке
                                                                                     //TIPS: Система должна иметь доступ к глобальным объектам.

//TIPS: Подсистема работает только во время своей функции run
pub type Resolver<State> = fn() -> State;

pub struct System<State> {
    pub state: State,
    pub estate: State,
    pub drawer: HashMap<State, Vec<Drawer>>,
    pub global: Rc<Mutex<DependencyMap>>,
    pub local: Rc<Mutex<DependencyMap>>,
    pub sub_system: HashMap<State, Box<dyn ExecSystem<dyn Any>>>,
    pub resolver: HashMap<State, Resolver<State>>,
}

impl<State: Clone> System<State> {
    fn new(istate: State, estate: State, global: Rc<Mutex<DependencyMap>>) -> Self {
        System {
            state: istate,
            estate,
            drawer: HashMap::new(),
            global: global.clone(),
            local: Rc::new(Mutex::new(DependencyMap::new())),
            sub_system: HashMap::new(),
            resolver: HashMap::new(),
        }
    }
}

pub trait ExecSystem<State: Sized + Eq + Hash> {
    fn add_drawer(&mut self, state: State, drawer: Drawer);
    fn set_resolver(&mut self, state: State, resolver: Resolver<State>);
    fn set_subsystem(&mut self, state: State, system: Box<dyn ExecSystem<dyn Any>>);
}
//TODO: Реализовать рекурсивную структуру ECS с возможностью подтипов

impl<State: Sized + Eq + Hash> ExecSystem<State> for System<State> {
    fn add_drawer(&mut self, state: State, drawer: Drawer) {
        if let Some(v) = self.drawer.get_mut(&state) {
            v.push(drawer);
        }
    }

    fn set_resolver(&mut self, state: State, resolver: Resolver<State>) {
        self.resolver.insert(state, resolver);
    }

    fn set_subsystem(&mut self, state: State, system: Box<dyn ExecSystem<dyn Any>>) {
        self.sub_system.insert(state, system);
    }
}

#[async_trait(?Send)]
pub trait ExecSystemDeps<State: Sized + Eq + Hash> {
    async fn add_local<T: Send + Sync + 'static>(&mut self, value: T);
    async fn get_local<V: Send + Sync + 'static>(&mut self) -> Arc<V>;
    async fn run<'a>(&mut self, f: &'a mut Terminal<CrosstermBackend<Stdout>>);
}

#[async_trait(?Send)]
impl<State: Sized + Eq + Hash> ExecSystemDeps<State> for System<State> {
    async fn add_local<T: Send + Sync + 'static>(&mut self, value: T) {
        self.local.lock().await.borrow_mut().insert(value);
    }

    async fn get_local<V: Send + Sync + 'static>(&mut self) -> Arc<V> {
        self.local.lock().await.borrow().get()
    }

    async fn run<'a>(&mut self, f: &'a mut Terminal<CrosstermBackend<Stdout>>) {}
}

async fn run<State: Sized + Eq + Hash>(
    slf: &mut System<State>,
    input: &Option<Event>,
    update: &Option<Update>,
) {
    let mut nu;
    if let Some(ss) = slf.sub_system.get(&slf.state) {

    } else {

    }
    let global = slf.global.lock().await;
    let mut rxk: Arc<mpsc::UnboundedReceiver<Event>> = global.get();
    let mut mc: Arc<Mutex<Client>> = global.get();
    let mut client = mc.lock().await.borrow_mut();
    nu = Box::pin(client.next_update());
    select(Box::pin(rxk.recv()), nu); //Как получать и ввод и обновления с минимальными задержками

    if let Some(vd) = slf.drawer.get(&slf.state) {
        let mut mt: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>> = global.get();
        let mut t = mt.lock().await.borrow_mut();
        let mut frame = t.get_frame();
        for d in vd.iter() {
            d(ArgumenDrawer {
                global: slf.global.clone(),
                local: slf.local.clone(),
                frame: &mut frame,
            });
        }
    }
    if let Some(r) = slf.resolver.get(&slf.state) {
        slf.state = r();
    }
}
