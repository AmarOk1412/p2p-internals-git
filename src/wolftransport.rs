// https://docs.rs/git2/0.13.17/git2/transport/fn.register.html

use git2::Error;
use git2::transport::SmartSubtransportStream;
use git2::transport::{Service, SmartSubtransport, Transport};

use std::io;
use std::io::prelude::*;

use bichannel::Channel;

use std::sync::{Arc, Mutex};

struct WolfTransport {
    channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>,
}

struct WolfSubTransport {
    action: Service,
    channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>,
    url: String,
    sent_request: bool
}


pub unsafe fn register(channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>) {
    git2::transport::register("wolf", move |remote| factory(remote, channel.clone())).unwrap();
}

fn factory(remote: &git2::Remote<'_>, channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>) -> Result<Transport, Error> {
    Transport::smart(
        remote,
        true,
        WolfTransport {
            channel
        },
    )
}

impl SmartSubtransport for WolfTransport {
    fn action(
        &self,
        url: &str,
        action: Service,
    ) -> Result<Box<dyn SmartSubtransportStream>, Error> {
        println!("Init subtransport");
        Ok(Box::new(WolfSubTransport {
            action,
            channel: self.channel.clone(),
            url: String::from(url),
            sent_request: false,
        }))
    }

    fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl Read for WolfSubTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        println!("READ");
        if !self.sent_request {
            // TODO
            let command = "upload-pack-ls TODO".as_bytes().to_vec();
            self.channel.lock().unwrap().send(command);
            self.sent_request = true;
        }
        Ok(self.channel.lock().unwrap().recv().unwrap_or(Vec::new()).len())
    }
}

impl Write for WolfSubTransport {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        println!("WRITE");
        if !self.sent_request {
            self.channel.lock().unwrap().send(data.to_vec());
            self.sent_request = true;
            // TODO
        }
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}