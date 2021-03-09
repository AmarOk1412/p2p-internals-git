mod wolftransport;
mod server;

use bichannel::channel;
use server::Server;
use tempfile::TempDir; // TODO remove
use std::thread;
use std::sync::{Arc, Mutex};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, Progress, RemoteCallbacks};

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
        let mut server = Server::new(Arc::new(Mutex::new(server_channel)), "TODO FROM");
        server.read();
    });

    let dest = TempDir::new().unwrap();



    let mut co = CheckoutBuilder::new();
    let mut fo = FetchOptions::new();
    RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone("wolf://localhost/zds", dest.path()).unwrap();

    server.join().expect("The sender thread has panicked");
}
