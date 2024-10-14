use tokio::runtime;

pub(crate) fn tokio_runtime() -> &'static runtime::Handle {
    use std::sync::OnceLock;
    use std::time::Duration;

    static RUNTIME: OnceLock<runtime::Handle> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio initialization error");
        let handle = rt.handle().clone();
        std::thread::spawn(move || {
            // Replace with the async main loop, or some sync structure to
            // control shutting it down if desired.
            rt.block_on(async {
                loop {
                    tokio::time::sleep(Duration::from_secs(10000)).await
                }
            });
        });
        handle
    })
}
