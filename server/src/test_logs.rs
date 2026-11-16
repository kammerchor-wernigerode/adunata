//! Captures what Adunata logs during tests, so tests can assert that no secret or personal data reaches the log.

#[derive(Clone, Default)]
pub struct CapturedLogs(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for CapturedLogs {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Captures the logs of every test in this binary. A thread-local subscriber would miss events whose callsites other test threads registered first.
pub fn captured_logs() -> &'static CapturedLogs {
    static LOGS: std::sync::OnceLock<CapturedLogs> = std::sync::OnceLock::new();
    LOGS.get_or_init(|| {
        let logs = CapturedLogs::default();
        let writer = logs.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("no other test may set a global subscriber");
        logs
    })
}

impl CapturedLogs {
    /// Everything logged so far by every test of this binary.
    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}
