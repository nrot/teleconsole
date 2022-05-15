use crate::ecs::System;
use crate::ecs::Key;
use crate::ecs::Drawer;
use crate::ecs::ArgumentDrawer;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum LoginState{
    PreLogin,
    EndLogin,
}


pub async fn DrawLogin<'a>(arg: ArgumentDrawer<'a>)->(){
    let mut f = arg.frame.lock().await;
    ()
}