use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc};
use crate::utils::Upstream;
use log::{info};

pub struct Transaction {
    pub client_addr: String,
    pub server_addr: String,
    pub begin_time: SystemTime,
    pub end_time: Duration,
    pub count: u32,
    pub cookie: String,
    pub success: bool,
    pub upstream: Arc<Upstream>,
}

impl Drop for Transaction {
    fn drop(&mut self) {
        self.end_time = self.begin_time.elapsed().unwrap();
        info!(
            "{}\t{}\t{}\t{}\t{}\t{}", self.begin_time.duration_since(UNIX_EPOCH).unwrap().as_millis(),
            self.client_addr, &self.server_addr, self.end_time.as_millis(), self.count, self.cookie
        );
        if self.server_addr == "-".to_string() {
            return;
        } else {
            let mut servers = self.upstream.servers.lock().unwrap();
            servers.free_addr(&self.server_addr, &self.cookie, self.success);
        }
    }
}