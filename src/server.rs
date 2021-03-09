
use bichannel::Channel;
use git2::Repository;
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
    repository: Repository,
    channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>,
    // socket: Socket, // TODO channel? tcp socket? other?
    wanted: String,
    common: String,
    haveReferences: Vec<String>,
    buf: Vec<u8>,
}

impl Server {
    pub fn new(channel: Arc<Mutex<Channel<Vec<u8>, Vec<u8>>>>, path: &str) -> Self {
        let repository = Repository::open("/home/amarok/Projects/tmp").unwrap();
        Self {
            path: path.to_string(),
            repository,
            channel,
            wanted: String::new(),
            common: String::new(),
            haveReferences: Vec::new(),
            buf: Vec::new(),
        }
    }

    pub fn read(&mut self) {
        loop {
            let buf = self.channel.lock().unwrap().recv().unwrap();
            self.onRecv(buf);
        }
    }

    fn onRecv(&mut self, buf: Vec<u8>) {
        let mut buf = Some(buf);
        let mut needMoreParsing = true;
        while needMoreParsing {
            needMoreParsing = self.parse(buf.take());
        }
    }

    fn parse(&mut self, mut buf: Option<Vec<u8>>) -> bool {
        // Parse pkt len
        // Reference: https://github.com/git/git/blob/master/Documentation/technical/protocol-common.txt#L51
        // The first four bytes define the length of the packet and 0000 is a FLUSH pkt
        if buf.is_some() {
            self.buf.append(&mut buf.unwrap());
        }
        let pkt_len = str::from_utf8(&self.buf[0..4]).unwrap();
        let pkt_len = i64::from_str_radix(pkt_len, 16).unwrap() as usize;
        let pkt : Vec<u8> = self.buf.drain(0..pkt_len).collect();
        let pkt = str::from_utf8(&pkt[0..pkt_len]).unwrap();
        println!("received pkt {}", pkt);


        if pkt.find(UPLOAD_PACK_CMD) == Some(4) {
            // Cf: https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L166
            // References discovery
            println!("Upload pack command detected");
            let parameters = Server::parameters(pkt);
            let version = match parameters.get("version") {
                Some(v) => v.parse::<i32>().unwrap_or(1),
                None => 1,
            };
            if version == 1 {
                self.send_references_capabilities(parameters.get("version").is_some());
            } else {
                println!("That protocol version is not yet supported (version: {})", version);
            }
        } else if pkt.find(WANT_CMD) == Some(4) {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L229
            // NOTE: a client may sends more than one want. Moreover, the first want line will sends
            // wanted capabilities such as `side-band-64, multi-ack, etc`. To simplify the code, we
            // just ignore capabilities & mutli-lines
            self.wanted = String::from(pkt.get(9..49).unwrap()); // take just the commit id
            println!("Detected wanted commit: {}", self.wanted);
        }

        //if pkt.find(UPLOAD_PACK_CMD) {
            // Cf: https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L166
            // References discovery
        //  println!("Upload pack command detected.");
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
        self.buf.len() != 0
    }

    fn send_references_capabilities(&self, send_version: bool) {
        // Answer with the version number
        // **** When the client initially connects the server will immediately respond
        // **** with a version number (if "version=1" is sent as an Extra Parameter),
        // TODO check unwrap()
        if send_version {
            let packet = "000eversion 1\0";
            self.channel.lock().unwrap().send(packet.as_bytes().to_vec()).unwrap();
        }

        let current_head = self.repository.refname_to_id("HEAD").unwrap();
        let mut capabilities = format!("{} HEAD\0side-band side-band-64k shallow no-progress include-tag", current_head);
        capabilities = format!("{:04x}{}\n", capabilities.len() + 5 /* size + \n */, capabilities);

        for name in self.repository.references().unwrap().names() {
            let reference: &str = name.unwrap();
            let oid = self.repository.refname_to_id(reference).unwrap();
            capabilities += &*format!("{:04x}{} {}\n", 6 /* size + space + \n */ + 40 /* oid */ + reference.len(), oid, reference);
        }

        print!("{}", capabilities);
        self.channel.lock().unwrap().send(capabilities.as_bytes().to_vec()).unwrap();
        println!("{}", FLUSH_PKT);
        self.channel.lock().unwrap().send(FLUSH_PKT.as_bytes().to_vec()).unwrap();
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

    fn parameters(pkt_line: &str) -> HashMap<String, String> {
        HashMap::new()
    }
}