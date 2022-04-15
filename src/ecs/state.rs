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

trait DynHash{
    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl Hash for dyn SizedSearch {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher {
        self.dyn_hash(state);
    }
}

impl<T> DynHash for T
where
    T: Hash {
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        self.hash(&mut state);
    }
}


impl PartialEq for SystemState{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) 
    }
}


pub struct SystemState(Arc<dyn SizedSearch>);

pub trait SizedSearch: Eq+ Send + Sync {}
impl<T: Eq + Send + Sync> SizedSearch for T {}