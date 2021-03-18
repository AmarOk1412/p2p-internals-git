// https://docs.rs/git2/0.13.17/git2/transport/fn.register.html

use bichannel::{ SendError, RecvError };
use git2::Error;
use git2::transport::SmartSubtransportStream;
use git2::transport::{Service, SmartSubtransport, Transport};
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

static HOST_TAG: &str = "host=";

// Note: this transport is useless, but is only here for an example.
// Every packet is predeceased by a header of 4 bytes: "WOLF".
pub struct WolfChannel
{
    pub channel: bichannel::Channel<Vec<u8>, Vec<u8>>,
}

impl WolfChannel
{
    pub fn recv(&self) -> Result<Vec<u8>, RecvError> {
        let res = self.channel.recv();
        if !res.is_ok() {
            return res;
        }
        let mut res = res.unwrap();
        res.drain(0..4);
        Ok(res)
    }

    pub fn send(&self, data: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        let mut to_send = "WOLF".as_bytes().to_vec();
        to_send.extend(data);
        self.channel.send(to_send)
    }
}

pub type Channel = Arc<Mutex<WolfChannel>>;

/**
 * Now, let's write a smart transport for git2-rs to answer to the scheme wolf://
 */
struct WolfTransport {
    channel: Channel,
}

struct WolfSubTransport {
    action: Service,
    channel: Channel,
    url: String,
    sent_request: bool
}

// git2 will use our smart transport for wolf://
pub unsafe fn register(channel: Channel) {
    git2::transport::register("wolf", move |remote| factory(remote, channel.clone())).unwrap();
}

fn factory(remote: &git2::Remote<'_>, channel: Channel) -> Result<Transport, Error> {
    Transport::smart(
        remote,
        false, // rpc = false, this means that our channel is connected during all the transaction.
        WolfTransport {
            channel
        },
    )
}

impl SmartSubtransport for WolfTransport {
    /**
     * Generate a new transport to use (because rpc = false), we will only answer to upload-pack-ls & receive-pack-ls
     */
    fn action(
        &self,
        url: &str,
        action: Service,
    ) -> Result<Box<dyn SmartSubtransportStream>, Error> {
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
    fn generate_request(cmd: &str, url: &str) -> Vec<u8> {
        // url format = wolf://host/repo
        // Note: This request is sent when the client's part is starting, to notify the server about what we want to do.
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
        // send a request when  starting
        if !self.sent_request {
            let cmd = match self.action {
                Service::UploadPackLs => "git-upload-pack",
                Service::UploadPack => "git-upload-pack",
                Service::ReceivePackLs => "git-receive-pack",
                Service::ReceivePack => "git-receive-pack",
            };
            let cmd = WolfTransport::generate_request(cmd, &*self.url);
            let _ = self.channel.lock().unwrap().send(cmd);
            self.sent_request = true;
        }
        // Write what the server sends into buf.
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
        let _ = self.channel.lock().unwrap().send(data.to_vec());
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        // Unused in our case
        Ok(())
    }
}