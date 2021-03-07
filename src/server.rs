
use bichannel::Channel;
use std::collections::HashMap;
use std::i64;
use std::str;
use std::sync::{ Arc, Mutex };

static FLUSH_PKT: &str = "0000";
static NAK_PKT: &str = "0008NAK\n";
static DONE_PKT: &str = "0009NAK\n";
static WANT_CMD: &str = "want";
static HAVE_CMD: &str = "have";
static UPLOAD_PACK_CMD: &str = "git-upload-pack";

pub struct Server {
    path: String,
    channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>,
    // socket: Socket, // TODO channel? tcp socket? other?
    wantedReference: String,
    common: String,
    haveReferences: Vec<String>,
    buf: Vec<u8>,
}

impl Server {
    pub fn new(channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>, path: &str) -> Self {
        Self {
            channel,
            path: path.to_string(),
            wantedReference: String::new(),
            common: String::new(),
            haveReferences: Vec::new(),
            buf: Vec::new(),
        }
    }

    pub fn read(&self) {
        loop {
            let buf = self.channel.lock().unwrap().recv().unwrap();
            self.onRecv(buf);
        }
    }

    fn onRecv(&self, buf: Vec<u8>) {
        let mut buf = Some(buf);
        let mut needMoreParsing = true;
        while needMoreParsing {
            needMoreParsing = self.parseOrder(buf.take());
        }
    }

    fn parseOrder(&self, mut buf: Option<Vec<u8>>) -> bool {
        // Parse pkt len
        // Reference: https://github.com/git/git/blob/master/Documentation/technical/protocol-common.txt#L51
        // The first four bytes define the length of the packet and 0000 is a FLUSH pkt
        let buf = buf.take().unwrap();
        let buf = str::from_utf8(&buf).unwrap();
        let pkt_len = i64::from_str_radix(buf.get(0..4).unwrap(), 16).unwrap();
        println!("received pkt_len {}", pkt_len);

        //if pkt.find(UPLOAD_PACK_CMD) {
            // Cf: https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L166
            // References discovery
        //  println!("Upload pack command detected.");
        // else if pkt.find(WANT_CMD) {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L229
        // else if pkt.find(HAVE_CMD) {
            // Detect first common commit
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L390
        // else if pkt.find(DONE_PKT) {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L390 Do
            // not do multi-ack, just send ACK + pack file
            // In case of no common base, send NAK
            // println!("Peer negotiation is done. Answering to want order");
        // else if pkt == FLUSH_PKT {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L390
            // Do not do multi-ack, just send ACK + pack file In case of no common base ACK
        true
    }

    fn sendReferenceCapabilities(&self, sendVersion: bool) {

    }

    fn answerToWantOrder(&self) {

    }

    fn NAK(&self) -> bool {
        self.channel.lock().unwrap().send(NAK_PKT.as_bytes().to_vec()).is_ok()
    }

    fn ACKCommon(&self) -> bool {
        let length = 18 /* size + ACK + space * 2 + continue + \n */ + self.common.len();
        let msg = format!("{:04x}ACK {} continue\n", length, self.common);
        self.channel.lock().unwrap().send(msg.as_bytes().to_vec()).is_ok()
    }

    fn ACKFirst(&self) -> bool {
        let length = 9 /* size + ACK + space + \n */ + self.common.len();
        let msg = format!("{:04x}ACK {}\n", length, self.common);
        self.channel.lock().unwrap().send(msg.as_bytes().to_vec()).is_ok()
    }

    fn sendPackData(&self) {

    }

    fn params(pkt_line: &String) -> HashMap<String, String> {
        HashMap::new()
    }
}