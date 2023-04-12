//! A thin wrapper around `ThreadPool` to make sure that we join all things
//! properly.
use crossbeam_channel::Sender;

pub(crate) struct TaskPool<T> {
    sender: Sender<T>,
    inner: threadpool::ThreadPool,
}

impl<T> TaskPool<T> {
    pub(crate) fn new_with_threads(sender: Sender<T>, threads: usize) -> TaskPool<T> {
        const STACK_SIZE: usize = 8 * 1024 * 1024;

        let inner = threadpool::Builder::new()
            .thread_name("Worker".into())
            .thread_stack_size(STACK_SIZE)
            .num_threads(threads)
            .build();
        TaskPool { sender, inner }
    }

    pub(crate) fn spawn<F>(&mut self, task: F)
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.inner.execute({
            let sender = self.sender.clone();
            move || sender.send(task()).unwrap()
        })
    }

    pub(crate) fn spawn_with_sender<F>(&mut self, task: F)
    where
        F: FnOnce(Sender<T>) + Send + 'static,
        T: Send + 'static,
    {
        self.inner.execute({
            let sender = self.sender.clone();
            move || task(sender)
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.inner.queued_count()
    }
}

impl<T> Drop for TaskPool<T> {
    fn drop(&mut self) {
        self.inner.join()
    }
}
