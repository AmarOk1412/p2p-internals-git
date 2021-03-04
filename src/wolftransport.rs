// https://docs.rs/git2/0.13.17/git2/transport/fn.register.html

use git2::Error;
use git2::transport::SmartSubtransportStream;
use git2::transport::{Service, SmartSubtransport, Transport};

use std::io;
use std::io::prelude::*;

struct WolfTransport {

}

struct WolfSubTransport {
    action: Service,
    url: String,
    sent_request: bool
}


pub unsafe fn register() {
    git2::transport::register("wolf", move |remote| factory(remote)).unwrap();
}

fn factory(remote: &git2::Remote<'_>) -> Result<Transport, Error> {
    Transport::smart(
        remote,
        true,
        WolfTransport {
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
        }
        Ok(0) // TODO
    }
}

impl Write for WolfSubTransport {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        println!("WRITE");
        if !self.sent_request {
            // TODO
        }
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}