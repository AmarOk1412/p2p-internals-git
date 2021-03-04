mod wolftransport;
mod server;

use server::Server;
use tempfile::TempDir; // TODO remove

fn main() {
    // TODO ./p2p-internal-git --repo FROM_DIR --dest DEST_DIR
    unsafe {
        wolftransport::register();
    }
    
    // TODO
    let server = Server::new("TODO FROM");

    let dest = TempDir::new().unwrap();
    let r = git2::Repository::clone("wolf://zds", dest.path()).unwrap();
}
