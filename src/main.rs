mod wolftransport;
mod server;

use bichannel::channel;
use server::Server;
use tempfile::TempDir; // TODO remove
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    // Usage: ./p2p-internal-git --repo FROM_DIR --dest DEST_DIR

    // Note: here we use a mpsc::channel for demo purposes, but
    // the transport can be on top of anything you want/need.
    // It can be replaced by a real server with TLS support for
    // example.
    let (server_channel, transport_channel) = channel();
    unsafe {
        wolftransport::register(Arc::new(Mutex::new(transport_channel)));
    }

    let server = thread::spawn(move || {
        let server = Server::new(Arc::new(Mutex::new(server_channel)), "TODO FROM");
        server.read();
    });

    let dest = TempDir::new().unwrap();
    let r = git2::Repository::clone("wolf://zds", dest.path()).unwrap();
    server.join().expect("The sender thread has panicked");
}
