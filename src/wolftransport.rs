// https://docs.rs/git2/0.13.17/git2/transport/fn.register.html

use bichannel::Channel;
use git2::Error;
use git2::transport::SmartSubtransportStream;
use git2::transport::{Service, SmartSubtransport, Transport};

use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

static HOST_TAG: &str = "host=";

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

impl WolfTransport {
    fn generateRequest(cmd: &str, url: &str) -> Vec<u8> {
        // url format = wolf://host/repo
        // Note: don't care about exception as it's just for a tuto
        let sep = url.rfind('/').unwrap();
        let host = url.get(7..sep).unwrap();
        let repo = url.get(sep..).unwrap();

        let null_char = '\0';
        let total = 4                                   /* 4 bytes for the len len */
                    + cmd.len()                         /* followed by the command */
                    + 1                                 /* space */
                    + repo.len()                        /* repo to clone */
                    + 1                                 /* \0 */
                    + HOST_TAG.len() + host.len()       /* host=server */
                    + 1                                 /* \0 */;
        let request = format!("{:04x}{} {}{}{}{}{}", total, cmd, repo, null_char, HOST_TAG, host, null_char);
        request.as_bytes().to_vec()
    }
}

impl Read for WolfSubTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        println!("READ");
        if !self.sent_request {
            let cmd = match self.action {
                Service::UploadPackLs => "git-upload-pack",
                Service::UploadPack => "git-upload-pack",
                Service::ReceivePackLs => "git-receive-pack",
                Service::ReceivePack => "git-receive-pack",
            };
            let cmd = WolfTransport::generateRequest(cmd, &*self.url);
            self.channel.lock().unwrap().send(cmd);
            self.sent_request = true;
        }
        let mut recv = self.channel.lock().unwrap().recv().unwrap_or(Vec::new());
        let mut iter = recv.drain(..);
        let mut idx = 0;
        while let Some(v) = iter.next() {
            buf[idx] = v;
            idx += 1;
        }
        Ok(idx)
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