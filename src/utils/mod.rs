use std::collections::{HashMap, LinkedList};
use std::sync::{Mutex};

pub struct Servers {
    pub addrs: Vec<String>,
    pub downs: LinkedList<String>,
    pub map: HashMap<String, (String, u32)>,
}

pub struct Upstream {
    pub servers: Mutex<Servers>,
}

impl Servers {
    pub fn get_addr(&mut self, cookie: &str) -> Result<(String, u32), std::io::Error> {
        if let Some((addr, count)) = self.map.get_mut(cookie) {
            let addr = addr.clone();
            *count = *count + 1;
            return Ok((addr, *count));
        }

        if self.addrs.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "no addr!"));
        }

        let addr = self.addrs.pop().unwrap();
        let raddr = addr.clone();
        self.map.insert(cookie.to_string(), (addr, 1));

        Ok((raddr, 1))
    }

    pub fn free_addr(&mut self, addr: &str, cookie: &str, success: bool) {
        if let Some((_, count)) = self.map.get_mut(cookie) {
            *count = *count - 1;
            if 0 == *count {//no addr
                self.map.remove(cookie);
                if success {//not success, example server connect failed
                    self.addrs.push(addr.to_string());
                } else {
                    self.downs.push_back(addr.to_string());
                }
            }
        }
    }
}