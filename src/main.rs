#[macro_use]
extern crate serde_derive;

use std::process;
use std::collections::{HashMap, LinkedList};
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};
use async_std::io;
use async_std::prelude::*;
use async_std::{task, stream, future};
use async_std::net::{TcpStream, TcpListener};
use log::{error};
mod conf;
mod logp;
mod trans;
mod utils;
use crate::utils::{Upstream, Servers};
use trans::Transaction;

const COOKIE_SIZE: usize = 37;

fn main() -> io::Result<()> {
    let _ = logp::init();

    let args = conf::new_args();
    let path = match args.get_path() {
        Some(p) => p,
        None => {
            error!("load config failed!");
            process::exit(1);
        }
    };

    let conf = conf::new_upstream(path)?;

    let listen_addr = conf.listen;
    let timeout = conf.timeout;

    let servers = Servers {
        addrs: conf.servers.clone(),
        downs: LinkedList::new(),
        map: HashMap::new(),
    };

    let upstreams = Arc::new(Upstream {
        servers: Mutex::new(servers),
    });

    println!("Listening on: {:?}", listen_addr);
    let down_upstream = upstreams.clone();
    task::spawn(async move {
        let mut interval = stream::interval(Duration::from_secs(10));
        let upstream = down_upstream.clone();
        while let Some(_) = interval.next().await {
            servers_helth_check(&upstream).await;
        }
    });

    task::block_on(async move {
        let upstream = upstreams.clone();
        server(listen_addr, upstream, timeout).await.unwrap();
    });

    Ok(())
}

async fn server(addr: String, upstreams: Arc<Upstream>, t: u32) -> io::Result<()> {
    let listen = TcpListener::bind(&addr).await?;
    let mut incoming = listen.incoming();
    while let Some(Ok(stream)) = incoming.next().await {
        let upstream = upstreams.clone();
        task::spawn(process(stream, upstream, t));
    }
    Ok(())
}

async fn process(mut inbound: TcpStream, upstream: Arc<Upstream>, t: u32) -> io::Result<()> {
    let mut trans = Transaction {
        client_addr: inbound.peer_addr()?.to_string(),
        server_addr: String::from("-"),
        begin_time: SystemTime::now(),
        end_time: Duration::from_millis(0),
        count: 0,
        cookie: String::from("-"),
        success: false,
        upstream: upstream.clone(),
    };

    let mut buf = vec![0u8; COOKIE_SIZE];
    let _n = inbound.read(&mut buf).await?;
    trans.cookie = String::from_utf8(buf.clone()).unwrap();
    {
        let mut servers = upstream.servers.lock().unwrap();
        let (addr, count) = servers.get_addr(&String::from_utf8(buf.clone()).unwrap())?;
        trans.server_addr = addr;
        trans.count = count;
    }
    let mut outbound = TcpStream::connect(trans.server_addr.clone()).await?;
    outbound.write_all(&buf).await?;

    let (mut ri, mut wi) = &mut (&inbound, &inbound);
    let (mut ro, mut wo) = &mut (&outbound, &outbound);

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);
    let handle = client_to_server.join(server_to_client);

    let _ = future::timeout(Duration::from_secs(t as u64), handle).await;

    trans.success = true;

    Ok(())
}

async fn servers_helth_check(upstream: &Arc<Upstream>) {
    let s =
        {
            let mut servers = upstream.servers.lock().unwrap();
            if servers.downs.is_empty() {
                None
            } else {
                servers.downs.pop_back()
            }
        };

    let s = match s {
        Some(t) => t,
        None => return,
    };

    if let Ok(_) = io::timeout(Duration::from_secs(5), TcpStream::connect(s.clone())).await {
        let mut servers = upstream.servers.lock().unwrap();
        servers.addrs.push(s);
    } else {
        let mut servers = upstream.servers.lock().unwrap();
        error!("check {} failed", &s);
        servers.downs.push_front(s);
    }
}