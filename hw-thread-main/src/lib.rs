#![deny(unsafe_code)]
use crate::sync::{Arc, Condvar, Mutex, thread};

#[cfg(feature = "loom")]
pub mod sync {
    pub use loom::sync::{Arc, Condvar, Mutex};
    pub use loom::thread;
}

#[cfg(not(feature = "loom"))]
pub mod sync {
    pub use std::sync::{Arc, Condvar, Mutex};
    pub use std::thread;
}

pub type Task = fn(i64);
// тут храним всё, что нужно защитить мьютексом(необработанные числа и флаг)
struct State {
    queue: Vec<i64>,
    shutting_down: bool,
}

// всё общее м/у потоками
// стейт, естественно переменную условия и таску, которую воркер дернет для каждого числа
struct Shared {
    state: Mutex<State>,
    has_work: Condvar,
    task: Task,
}

pub struct ThreadPool {
    shared: Arc<Shared>,                  // база
    workers: Vec<thread::JoinHandle<()>>, // для шатдаун
}

impl ThreadPool {
    pub fn new(worker_count: usize, task: Task) -> Self {
        // паникуем если worker_count == 0
        if worker_count == 0 {
            panic!("worker_count must be greater than 0");
        }
        // инциализируемся
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                queue: Vec::new(),
                shutting_down: false,
            }),
            has_work: Condvar::new(),
            task: task,
        });

        // запускаем потоки и кажому даёём своего клона(все будут иметь разные ссылки на одну память в куче, т.к. Arc создаёт на куче)
        let workers = (0..worker_count)
            .map(|_| {
                let shared = Arc::clone(&shared); // клонируем как в доке, явненько
                thread::spawn(move || worker_loop(shared))
            })
            .collect();

        ThreadPool { shared, workers }
    }

    pub fn execute(&self, num: i64) {
        let mut state = self.shared.state.lock().expect("mutex poisoned"); // база, только 1 поток меняет очередь
        state.queue.push(num);
        // будим одного слейва, т.к. 1 задача
        self.shared.has_work.notify_one();
        // когда выполнится функция, то мьютекс освободится, т.к. покинем область видимости
    }

    // завершаем все таски и стопаем всех слейвов
    pub fn shutdown(self) {
        // ультра важная штука {}, этот блок отпускает мьютекс до джоин, чтобы слейвы(воркеры) могли работать
        {
            let mut state = self.shared.state.lock().expect("mutex poisoned");
            state.shutting_down = true;
            // notify_all: на Condvar могут спать несколько workers одновременно.
            self.shared.has_work.notify_all();
        }

        // дожидается завершения каждого воркера, включая задачи из очереди
        // пока все не отработают и не выйдут не будет возвращён шатдаун
        for worker in self.workers {
            worker.join().expect("worker panicked");
        }
    }
}
// держим мьютекс только для работы с очередью, таску выполняем после дропа мьютекса
fn worker_loop(shared: Arc<Shared>) {
    loop {
        let maybe_num = {
            let mut state = shared.state.lock().expect("mutex poisoned");

            while state.queue.is_empty() && !state.shutting_down {
                state = shared
                    .has_work
                    .wait(state)
                    .expect("mutex poisoned while waiting");
            }

            if let Some(num) = state.queue.pop() {
                Some(num)
            } else if state.shutting_down {
                None
            } else {
                continue;
            }
        };
        match maybe_num {
            Some(num) => (shared.task)(num),
            None => break,
        }
    }
}
