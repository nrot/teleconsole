pub mod state;

use std::{
    borrow::{Borrow, BorrowMut},
    collections::{HashMap, BTreeMap},
    io::Stdout,
    pin::Pin,
    rc::Rc,
    sync::Arc, any::Any, hash::Hash,
};

use async_trait::async_trait;
use crossterm::event::Event;
use dptree::di::{DependencyMap, DependencySupplier};
use futures::Future;
use grammers_client::types::Update;
use tokio::sync::Mutex;
use tui::{backend::CrosstermBackend, Frame, Terminal};

use self::state::SystemState;

pub struct ArgumenDrawer<'a> {
    pub frame: Rc<Mutex<Frame<'a, CrosstermBackend<Stdout>>>>,
    pub global: Rc<Mutex<DependencyMap>>,
    pub local: Rc<Mutex<DependencyMap>>,
    pub inputs: Option<Event>,
    pub events: Rc<Mutex<Option<Update>>>,
}

pub type Drawer = fn(arguments: ArgumenDrawer) -> Pin<Box<dyn Future<Output = ()>>>; //TODO: Переделать в динамические аргументы ?????
                                                                                     //TIPS: Продумать как передовать всю необходимую информацию во внутрь систем по цепочке
                                                                                     //TIPS: Система должна иметь доступ к глобальным объектам.

//TIPS: Подсистема работает только во время своей функции run
pub type Resolver<State> =
    fn(system: &mut System<State>, input: Option<Event>, update: Rc<Mutex<Option<Update>>>) -> State;


#[derive(Debug, PartialEq, Eq)]
pub enum RunState {
    Tick,
    Ready,
}


pub type SystemId = usize;
pub type SystemList = HashMap<SystemId, System<dyn SystemState>>;

pub struct System<State> {
    pub id: SystemId,
    pub state: State,
    pub estate: State,
    pub drawer: BTreeMap<State, Vec<Drawer>>,
    pub global: Rc<Mutex<DependencyMap>>,
    pub local: Rc<Mutex<DependencyMap>>,
    pub sub_system: BTreeMap<State, SystemId>,
    pub resolver: BTreeMap<State, Resolver<State>>,
}

impl<State> Hash for System<State> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<State: Ord> System<State> {
    fn new(id: SystemId, istate: State, estate: State, global: Rc<Mutex<DependencyMap>>) -> Self {
        System {
            id,
            state: istate,
            estate,
            drawer: BTreeMap::new(),
            global,
            local: Rc::new(Mutex::new(DependencyMap::new())),
            sub_system: BTreeMap::new(),
            resolver: BTreeMap::new(),
        }
    }
}

pub trait ExecSystem<State: Ord>:
    ExecSystemLocals<State> + ExecSystemDeps<State>
{
}
impl<T: ExecSystemLocals<State> + ExecSystemDeps<State>, State: Ord>
    ExecSystem<State> for T
{
}

pub trait ExecSystemLocals<State: Ord> {
    fn add_drawer(&mut self, state: State, drawer: Drawer);
    fn set_resolver(&mut self, state: State, resolver: Resolver<State>);
    fn set_subsystem(&mut self, state: State, system: SystemId);
    fn get_subsystem_of_state(&self) -> Option<SystemId>;
    fn get_drawer_of_state(&self)->Vec<Drawer>;
    fn get_resolver_of_state(&self)->Option<Resolver<State>>;
}
//TODO: Реализовать рекурсивную структуру ECS с возможностью подтипов

impl<State: Ord> ExecSystemLocals<State> for System<State> {
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
    fn get_subsystem_of_state(&self) -> Option<SystemId> {
        self.sub_system.get(&self.state).cloned()
    }

    fn get_drawer_of_state(&self)->Vec<Drawer>{
        match self.drawer.get(&self.state).cloned(){
            Some(l) => l,
            None => Vec::new(),
        }
    }

    fn get_resolver_of_state(&self)->Option<Resolver<State>> {
        self.resolver.get(&self.state).cloned()
    }
    
}

#[async_trait(?Send)]
pub trait ExecSystemDeps<State> {
    async fn add_local<T: Send + Sync + 'static>(&mut self, value: T);
    async fn get_local<V: Send + Sync + 'static>(&mut self) -> Arc<V>;
    fn run(&mut self, input: Option<Event>, events: Rc<Mutex<Option<Update>>>) -> RunState;
}

#[async_trait(?Send)]
impl<State: Ord> ExecSystemDeps<State> for System<State> {
    async fn add_local<T: Send + Sync + 'static>(&mut self, value: T) {
        self.local.lock().await.borrow_mut().insert(value);
    }

    async fn get_local<V: Send + Sync + 'static>(&mut self) -> Arc<V> {
        self.local.lock().await.borrow().get()
    }

    fn run(&mut self, input: Option<Event>, events: Rc<Mutex<Option<Update>>>) -> RunState {
        let resolver = self.get_resolver_of_state().expect("Not have resolver for system");
        let nstate = resolver(self, input, events);
        if self.estate == nstate{
            self.state = nstate;
            RunState::Ready
        } else {
            RunState::Tick
        }
    }
}

///TODO: Избавиться от рекурсии последовательным опросом потомков через get_subsystem_of_state(&self)->SystemId и созданием стека
async fn step<State: Ord>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    systems: &mut SystemList,
    input: Option<Event>,
    update: Option<Update>,
) -> bool {
    let mut stack = vec![0];

    while let Some(system) = systems.get(stack.last().unwrap()) {
        match system.get_subsystem_of_state() {
            Some(s) => stack.push(s),
            None => break,
        }
    }

    let mut system = systems.get_mut(&stack.pop().unwrap()).unwrap();
    let frame = Rc::new(Mutex::new(terminal.get_frame()));
    let events = Rc::new(Mutex::new(update));
    
    for drawer in system.get_drawer_of_state().iter(){
        drawer(ArgumenDrawer{
            events: events.clone(),
            frame: frame.clone(),
            inputs: input,
            global: system.global.clone(),
            local: system.local.clone()
        }).await;
    }

    while RunState::Ready == system.run(input, events.clone()){
        if let Some(s) = stack.pop(){
            system = systems.get_mut(&s).unwrap();
        } else {
            break;
        }
    }
    !stack.is_empty()
    

    // if let Some(ss) = slf.sub_system.get(&slf.state) {
    //     if let Some(ss) = systems.get_mut(ss) {
    //         match *run(ss, systems, input, update).await {
    //             RunState::Ready(Ok(_)) => {}
    //             RunState::Tick => {}
    //             RunState::Ready(Err(e)) => {}
    //         }
    //     }
    // }
    // let global = slf.global.lock().await;
    // let mut rxk: Arc<mpsc::UnboundedReceiver<Event>> = global.get();
    // let mut mc: Arc<Mutex<Client>> = global.get();
    // let mut client = mc.lock().await.borrow_mut();
    // nu = Box::pin(client.next_update());
    // select(Box::pin(rxk.recv()), nu); //Как получать и ввод и обновления с минимальными задержками

    // if let Some(vd) = slf.drawer.get(&slf.state) {
    //     let mut mt: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>> = global.get();
    //     let mut t = mt.lock().await.borrow_mut();
    //     let mut frame = t.get_frame();
    //     for d in vd.iter() {
    //         d(ArgumenDrawer {
    //             global: slf.global.clone(),
    //             local: slf.local.clone(),
    //             frame: &mut frame,
    //             inputs: input,
    //             events: update,
    //         });
    //     }
    // }
    // if let Some(r) = slf.resolver.get(&slf.state) {
    //     slf.state = r();
    // };
}
