use synchronization_thread_pool::ThreadPool;

fn noop_task(_: i64) {}

#[test]
fn loom_shutdown_wakes_idle_worker() {
    loom::model(|| {
        let pool = ThreadPool::new(1, noop_task);
        pool.shutdown();
    });
}

#[test]
fn loom_submitted_work_does_not_deadlock_shutdown() {
    loom::model(|| {
        let pool = ThreadPool::new(1, noop_task);

        pool.execute(10);
        pool.execute(20);
        pool.shutdown();
    });
}

#[test]
fn loom_submitter_can_finish_before_shutdown() {
    loom::model(|| {
        let pool = loom::sync::Arc::new(ThreadPool::new(1, noop_task));

        let submitter = {
            let pool = pool.clone();
            loom::thread::spawn(move || pool.execute(30))
        };

        submitter.join().expect("submitter panicked");

        let pool = loom::sync::Arc::try_unwrap(pool)
            .ok()
            .expect("pool still shared");
        pool.shutdown();
    });
}
