use std::{collections::HashMap, rc::Rc, any::Any};

use tokio::sync::Mutex;

#[derive(Default, Clone)]
pub struct DependencyMap {
    map: HashMap<&'static str, Dependency>,
}

#[derive(Clone)]
pub struct Dependency {
    inner: Rc<Mutex<dyn Any + Send + Sync>>,
}

impl Dependency{
    pub fn new(item: Box<dyn Any + Send + Sync>)->Self{
        Dependency{
            inner: Rc::new(Mutex::new(item))
        }
    }
    pub fn inner(&self)->Rc<Mutex<dyn Any + Send + Sync>>{
        self.inner.clone()
    }
}

impl PartialEq for DependencyMap {
    fn eq(&self, other: &Self) -> bool {
        let keys1 = self.map.keys();
        let keys2 = other.map.keys();
        keys1.zip(keys2).map(|(k1, k2)| k1 == k2).all(|x| x)
    }
}

impl DependencyMap{
    pub fn new()->Self{
        DependencyMap { map: HashMap::new() }
    }

    pub fn insert(&mut self, key: &'static str, item: Dependency)->Option<Dependency>{
        self.map.insert(key, item)
    }
    pub fn insert_container(&mut self, container: Self){
        self.map.extend(container.map);
    }
    pub fn remove(&mut self, key: &'static str)->Option<Dependency>{
        self.map.remove(key)
    }
    pub fn get(&self, key: &'static str)->Option<Dependency>{
        self.map.get(key).cloned()
    }
}