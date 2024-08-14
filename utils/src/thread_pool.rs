use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ScopedThreadPool<'scope> {
    #[allow(dead_code)]
    workers: Vec<ScopedWorker<'scope>>,
    sender: mpsc::Sender<ScopedJob<'scope>>,
}

type ScopedJob<'scope> = Box<dyn FnOnce() + Send + 'scope>;

impl<'scope> ScopedThreadPool<'scope> {
    pub fn new<'env>(
        size: usize,
        scope: &'scope thread::Scope<'scope, 'env>,
    ) -> ScopedThreadPool<'scope> {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(ScopedWorker::new(id, receiver.clone(), scope));
        }

        ScopedThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct ScopedWorker<'scope> {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: thread::ScopedJoinHandle<'scope, ()>,
}

impl<'scope> ScopedWorker<'scope> {
    fn new<'env>(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<ScopedJob<'scope>>>>,
        thread_scope: &'scope thread::Scope<'scope, 'env>,
    ) -> ScopedWorker<'scope> {
        let thread = thread_scope.spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            if job.is_err() {
                break;
            }

            job.unwrap()();
        });

        ScopedWorker { id, thread }
    }
}
