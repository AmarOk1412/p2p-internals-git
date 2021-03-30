mod wolftransport;
mod server;

use bichannel::channel;
use wolftransport::WolfChannel;
use git2::build::RepoBuilder;
use server::Server;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: ./p2p-internal-git <src_dir> <dest_dir>");
        return;
    }
    
    let src_dir = args[1].clone();
    let dest_dir = Path::new(&args[2]);

    // For fetch, comment the 4 following lines
    if dest_dir.is_dir() {
        println!("Can't clone into an existing directory");
        return;
    }

    // Note: here we use a mpsc::channel for demo purposes, but
    // the transport can be on top of anything you want/need.
    // It can be replaced by a real server with TLS support for
    // example.
    let (server_channel, transport_channel) = channel();
    let transport_channel = Arc::new(Mutex::new(WolfChannel {
        channel: transport_channel
    }));
    let server_channel = Arc::new(Mutex::new(WolfChannel {
        channel: server_channel
    }));
    unsafe {
        wolftransport::register(transport_channel);
    }

    let server = thread::spawn(move || {
        println!("Starting server for {}", src_dir);
        let mut server = Server::new(server_channel, &*src_dir);
        server.run();
    });

    // For fetch
    // let repository = git2::Repository::open(dest_dir).unwrap();
    // let mut remote = repository.remote_anonymous("wolf://localhost/zds").unwrap();
    // let mut fo = git2::FetchOptions::new();
    // remote.fetch(&[] as &[&str], Some(&mut fo), None).unwrap();

    // For clone
    RepoBuilder::new().clone("wolf://localhost/zds", dest_dir).unwrap();
    // Note: "wolf://" triggers our registered transport. localhost/zds is unused
    // as our server only serves one repository and the address is not resolved.
    println!("Cloned into {:?}!", dest_dir);

    server.join().expect("The server panicked");
}
