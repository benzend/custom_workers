use std::{
    any::Any,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct WorkerGroup<T> {
    workers: Arc<Mutex<Vec<Worker<T>>>>,
}

impl<T> WorkerGroup<T> {
    pub fn new() -> WorkerGroup<T> {
        WorkerGroup {
            workers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn add(&mut self, worker: Worker<T>) {
        if let Ok(mut workers) = self.workers.lock() {
            workers.push(worker)
        }
    }
    pub fn join_workers(&mut self) -> Vec<WorkerJoinResult<T, Box<dyn Any + Send>>> {
        let mut results: Vec<WorkerJoinResult<T, Box<dyn Any + Send>>> = Vec::new();
        if let Ok(mut workers) = self.workers.lock() {
            while workers.len() > 0 {
                let worker = workers.pop();
                if let Some(w) = worker {
                    if let Some(h) = w.handle {
                        match h.join() {
                            Ok(res) => results.push(WorkerJoinResult {
                                error: None,
                                result: Some(res),
                            }),
                            Err(err) => results.push(WorkerJoinResult {
                                error: Some(err),
                                result: None,
                            }),
                        }
                    }
                }
            }
        }
        results
    }
}

pub struct Worker<T> {
    pub handle: Option<JoinHandle<T>>,
}

impl<T: Send + 'static> Worker<T> {
    pub fn new() -> Worker<T> {
        Worker { handle: None }
    }
    pub fn new_job(&mut self, f: fn() -> T) {
        self.handle = Some(thread::spawn(move || f()))
    }
}

pub struct WorkerJoinResult<T, E> {
    pub result: Option<T>,
    pub error: Option<E>,
}

mod tests {
    #[test]
    fn single_worker() {
        use crate::workers::{Worker, WorkerGroup};

        let workers = vec![Worker::new(|| "yay I work")];
        let mut worker_group = WorkerGroup::new();
        for worker in workers {
            worker_group.add(worker);
        }
        let results = worker_group.join_workers();
        assert_eq!(results[0].result, Some("yay I work"))
    }

    #[test]
    fn multiple_workers() {
        use crate::workers::{Worker, WorkerGroup};

        let workers = vec![
            Worker::new(|| "yay I work"),
            Worker::new(|| "but did I really?"),
            Worker::new(|| "I mean I guess"),
        ];
        let mut worker_group = WorkerGroup::new();
        for worker in workers {
            worker_group.add(worker);
        }
        let results = worker_group.join_workers();
        let mut yay_i_work_fullfilled = false;
        let mut but_did_i_really = false;
        let mut i_mean_i_guess = false;

        for r in results {
            match r.result {
                Some(msg) => match msg {
                    "yay I work" => yay_i_work_fullfilled = true,
                    "but did I really?" => but_did_i_really = true,
                    "I mean I guess" => i_mean_i_guess = true,
                    _ => panic!("Test failed"),
                },
                _ => panic!("Test failed"),
            }
        }

        assert_eq!(
            (true, true, true),
            (yay_i_work_fullfilled, but_did_i_really, i_mean_i_guess)
        )
    }

    #[test]
    fn multiple_async_workers() {
        use crate::workers::{Worker, WorkerGroup};
        use std::thread;
        use std::time::Duration;

        let workers = vec![
            Worker::new(|| {
                thread::sleep(Duration::from_secs(1));
                "yay I work"
            }),
            Worker::new(|| {
                thread::sleep(Duration::from_secs(1));
                "but did I really?"
            }),
            Worker::new(|| {
                thread::sleep(Duration::from_secs(1));
                "I mean I guess"
            }),
        ];
        let mut worker_group = WorkerGroup::new();
        for worker in workers {
            worker_group.add(worker);
        }
        let results = worker_group.join_workers();
        let mut yay_i_work_fullfilled = false;
        let mut but_did_i_really = false;
        let mut i_mean_i_guess = false;

        for r in results {
            match r.result {
                Some(msg) => match msg {
                    "yay I work" => yay_i_work_fullfilled = true,
                    "but did I really?" => but_did_i_really = true,
                    "I mean I guess" => i_mean_i_guess = true,
                    _ => panic!("Test failed"),
                },
                _ => panic!("Test failed"),
            }
        }

        assert_eq!(
            (true, true, true),
            (yay_i_work_fullfilled, but_did_i_really, i_mean_i_guess)
        )
    }

    #[test]
    fn works_with_strings() {
        use crate::workers::{Worker, WorkerGroup};

        let workers = vec![
            Worker::new(|| "yay I work".to_string()),
            Worker::new(|| "but did I really?".to_string()),
            Worker::new(|| "I mean I guess".to_string()),
        ];
        let mut worker_group = WorkerGroup::new();
        for worker in workers {
            worker_group.add(worker);
        }
        let results = worker_group.join_workers();
        let mut yay_i_work_fullfilled = false;
        let mut but_did_i_really = false;
        let mut i_mean_i_guess = false;

        for r in results {
            match r.result {
                Some(msg) => match msg.as_str() {
                    "yay I work" => yay_i_work_fullfilled = true,
                    "but did I really?" => but_did_i_really = true,
                    "I mean I guess" => i_mean_i_guess = true,
                    _ => panic!("Test failed"),
                },
                _ => panic!("Test failed"),
            }
        }

        assert_eq!(
            (true, true, true),
            (yay_i_work_fullfilled, but_did_i_really, i_mean_i_guess)
        )
    }
}
