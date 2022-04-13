use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    collections::{HashMap, HashSet},
    hash::Hash,
    io::Stdout,
    pin::Pin,
    rc::Rc,
    sync::Arc, error::Error, process::Output,
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

pub enum RunState{
    Poll,
    Ready(Result<(), Box<dyn Error>>)
}

pub type SystemId = usize;
pub type SystemList<State> = HashMap<SystemId, System<State>>;

pub struct System<State> {
    pub id: SystemId,
    pub state: State,
    pub estate: State,
    pub drawer: HashMap<State, Vec<Drawer>>,
    pub global: Rc<Mutex<DependencyMap>>,
    pub local: Rc<Mutex<DependencyMap>>,
    pub sub_system: HashMap<State, SystemId>,
    pub resolver: HashMap<State, Resolver<State>>,
}

impl<State> Hash for System<State>{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<State: Clone> System<State> {
    fn new(id: SystemId, istate: State, estate: State, global: Rc<Mutex<DependencyMap>>) -> Self {
        System {
            id,
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

pub trait SizedSearch: Eq + Hash{}
impl<T: Eq + Hash> SizedSearch for T{}

pub trait ExecSystem<State: SizedSearch, Output>: ExecSystemLocals<State> + ExecSystemDeps<State, Output>{}
impl<T: ExecSystemLocals<State> + ExecSystemDeps<State, Output>, State: SizedSearch, Output> ExecSystem<State, Output> for T{}

pub trait ExecSystemLocals<State: Sized + Eq + Hash> {
    fn add_drawer(&mut self, state: State, drawer: Drawer);
    fn set_resolver(&mut self, state: State, resolver: Resolver<State>);
    fn set_subsystem(&mut self, state: State, system: SystemId);
}
//TODO: Реализовать рекурсивную структуру ECS с возможностью подтипов

impl<State: SizedSearch> ExecSystemLocals<State> for System<State> {
    fn add_drawer(&mut self, state: State, drawer: Drawer) {
        if let Some(v) = self.drawer.get_mut(&state) {
            v.push(drawer);
        }
    }

    fn set_resolver(&mut self, state: State, resolver: Resolver<State>) {
        self.resolver.insert(state, resolver);
    }

    fn set_subsystem(&mut self, state: State, system: SystemId) {
        self.sub_system.insert(state, system);
    }
}

#[async_trait(?Send)]
pub trait ExecSystemDeps<State: Sized + Eq + Hash, Output: Sized> {
    async fn add_local<T: Send + Sync + 'static>(&mut self, value: T);
    async fn get_local<V: Send + Sync + 'static>(&mut self) -> Arc<V>;
    async fn run<'a>(&mut self, f: &'a mut Terminal<CrosstermBackend<Stdout>>)->RunState;
}

#[async_trait(?Send)]
impl<State: Sized + Eq + Hash, Output: Sized> ExecSystemDeps<State, Output> for System<State> {
    async fn add_local<T: Send + Sync + 'static>(&mut self, value: T) {
        self.local.lock().await.borrow_mut().insert(value);
    }

    async fn get_local<V: Send + Sync + 'static>(&mut self) -> Arc<V> {
        self.local.lock().await.borrow().get()
    }

    async fn run<'a>(&mut self, f: &'a mut Terminal<CrosstermBackend<Stdout>>)->RunState {
        todo!();
    }
}

///TODO: Избавиться от рекурсии последовательным опросом потомков через get_subsystem_of_state(&self)->SystemId и созданием стека
async fn run<State: Eq + Hash>(
    slf: &mut System<State>,
    systems: &SystemList<State>,
    input: &Option<Event>,
    update: &Option<Update>,
) -> Box<RunState> {
    let mut nu;
    if let Some(ss) = slf.sub_system.get(&slf.state) {
        if let Some(ss) = systems.get_mut(ss){
            match *run(ss, systems, input, update).await{
                RunState::Ready(Ok(_))=>{
    
                },
                RunState::Poll => {return Box::new(RunState::Poll)},
                RunState::Ready(Err(e))=>{
                    return Box::new(RunState::Ready(Err(e)))
                }
            }
        }
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
                inputs: input,
                events: update
            });
        }
    }
    if let Some(r) = slf.resolver.get(&slf.state) {
        slf.state = r();
    };
    Box::new(RunState::Poll)
}
