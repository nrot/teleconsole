/*TODO Сделать процедурный макрос для состяний, который будет делать object-safe Eq и Hash
План:
1. Делаем трейт для новых безопстных функций
2. Объявляем что для него реализованы Eq и Hash
3. В SystemState<Arc< используем новый трейт >>
4. В ExecSystemLocals Используем Eq и Hash
5. Процедурный Макрос который реализует новый трейт для структуры

Сделать возможность любой структуры быть системой и отказаться
от использования локальных переменных в сторону полей структуры.
Для этого нужен трейт System и процедурный макрос который реализует этот трейт
*/

use std::{any::{Any, TypeId}, hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};


pub trait Key {
    fn eq(&self, other: &dyn Key) -> bool;
    fn hash(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Eq + Hash + 'static> Key for T {
    fn eq(&self, other: &dyn Key) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            return self == other;
        }
        false
    }

    fn hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        // mix the typeid of T into the hash to make distinct types
        // provide distinct hashes
        Hash::hash(&(TypeId::of::<T>(), self), &mut h);
        h.finish()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for Box<dyn Key> {
    fn eq(&self, other: &Self) -> bool {
        Key::eq(self.as_ref(), other.as_ref())
    }
}

impl Eq for Box<dyn Key> {}

impl Hash for Box<dyn Key> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let key_hash = Key::hash(self.as_ref());
        state.write_u64(key_hash);
    }
}

///Depreceted
pub(crate) fn into_key(key: impl Eq + Hash + 'static) -> Box<dyn Key> {
    Box::new(key)
}