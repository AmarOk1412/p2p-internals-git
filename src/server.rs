use crate::wolftransport::Channel;
use git2::{ Buf, Oid, Repository, Sort };
use std::cmp::{ max, min };
use std::i64;
use std::str;

static FLUSH_PKT: &str = "0000";
static NAK_PKT: &str = "0008NAK\n";
static DONE_PKT: &str = "0009done\n";
static WANT_CMD: &str = "want";
static HAVE_CMD: &str = "have";
static UPLOAD_PACK_CMD: &str = "git-upload-pack";

/**
 * Represents a git server working a our custom transport, serving a given repository
 */
pub struct Server {
    repository: Repository,
    channel: Channel,
    wanted: String,
    common: String,
    have: Vec<String>,
    buf: Vec<u8>,
    stop: bool,
}

impl Server {
    /**
     * Creates a new Server for a given path and transport
     * @param channel       Our custom transport
     * @param path          Repository
     * @return the server
     */
    pub fn new(channel: Channel, path: &str) -> Self {
        let repository = Repository::open(path).unwrap();
        Self {
            repository,
            channel,
            wanted: String::new(),
            common: String::new(),
            have: Vec::new(),
            buf: Vec::new(),
            stop: false,
        }
    }

    pub fn run(&mut self) {
        // stop is set to true when the clone finished.
        // until then, answer to the client.
        while !self.stop {
            let buf = self.channel.lock().unwrap().recv().unwrap();
            self.recv(buf);
        }
    }

    fn recv(&mut self, buf: Vec<u8>) {
        let mut buf = Some(buf);
        let mut need_more_parsing = true;
        // Then client can send multiple pkt-lines in one buffer,
        // so, parse until ready to receive new orders.
        while need_more_parsing {
            need_more_parsing = self.parse(buf.take());
        }
    }

    fn parse(&mut self, buf: Option<Vec<u8>>) -> bool {
        // Parse pkt len
        // Reference: https://github.com/git/git/blob/master/Documentation/technical/protocol-common.txt#L51
        // The first four bytes define the length of the packet and 0000 is a FLUSH pkt
        if buf.is_some() {
            self.buf.append(&mut buf.unwrap());
        }
        let pkt_len = str::from_utf8(&self.buf[0..4]).unwrap();
        let pkt_len = max(4 as usize, i64::from_str_radix(pkt_len, 16).unwrap() as usize);
        let pkt : Vec<u8> = self.buf.drain(0..pkt_len).collect();
        let pkt = str::from_utf8(&pkt[0..pkt_len]).unwrap();
        println!("RECV: {}", pkt);

        if pkt.find(UPLOAD_PACK_CMD) == Some(4) {
            // Cf: https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L166
            // References discovery
            println!("Upload pack command detected");
            // NOTE: the upload-pack command can contains some parameters like version=1
            // For now git supports only version=1 so we can ignore this part for this article.
            self.send_references_capabilities();
        } else if pkt.find(WANT_CMD) == Some(4) {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L229
            // NOTE: a client may sends more than one want. Moreover, the first want line will sends
            // wanted capabilities such as `side-band-64, multi-ack, etc`. To simplify the code, we
            // just ignore capabilities & mutli-lines
            self.wanted = String::from(pkt.get(9..49).unwrap()); // take just the commit id
            println!("Detected wanted commit: {}", self.wanted);
        } else if pkt.find(HAVE_CMD) == Some(4) {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L390
            // NOTE: improve this part for multi-ack
            let have_commit = String::from(pkt.get(9..49).unwrap()); // take just the commit id
            if self.common.is_empty() {
                if self.repository.find_commit(Oid::from_str(&*have_commit).unwrap()).is_ok() {
                    self.common = have_commit.clone();
                    println!("Set common commit to: {}", self.common);
                }
            }
            self.have.push(have_commit);
        } else if pkt == DONE_PKT {
            // Reference:
            // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L390
            // NOTE: Do not do multi-ack, just send ACK + pack file
            // In case of no common base, send NAK
            println!("Peer negotiation is done. Answering to want order");
            let send_data = match self.common.is_empty() {
                true => self.nak(),
                false => self.ack_first(),
            };
            if send_data {
                self.send_pack_data();
            }
            self.stop = true;
        } else if pkt == FLUSH_PKT {
            if !self.have.is_empty() {
                // Reference:
                // https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L390
                // NOTE: Do not do multi-ack, just send ACK + pack file In case of no common base ACK
                self.ack_common();
                self.nak();
            } else if self.wanted.is_empty() {
                self.stop = true;
            }
        } else {
            println!("Unwanted packet received: {}", pkt);
        }
        self.buf.len() != 0
    }

    fn send_references_capabilities(&self) {
        let current_head = self.repository.refname_to_id("HEAD").unwrap();
        let mut capabilities = format!("{} HEAD\0side-band side-band-64k shallow no-progress include-tag multi_ack", current_head);
        capabilities = format!("{:04x}{}\n", capabilities.len() + 5 /* size + \n */, capabilities);

        for name in self.repository.references().unwrap().names() {
            let reference: &str = name.unwrap();
            let oid = self.repository.refname_to_id(reference).unwrap();
            capabilities += &*format!("{:04x}{} {}\n", 6 /* size + space + \n */ + 40 /* oid */ + reference.len(), oid, reference);
        }

        print!("Send: {}", capabilities);
        self.channel.lock().unwrap().send(capabilities.as_bytes().to_vec()).unwrap();
        println!("Send: {}", FLUSH_PKT);
        self.channel.lock().unwrap().send(FLUSH_PKT.as_bytes().to_vec()).unwrap();
    }

    fn nak(&self) -> bool {
        print!("Send: {}", NAK_PKT);
        self.channel.lock().unwrap().send(NAK_PKT.as_bytes().to_vec()).is_ok()
    }

    fn ack_common(&self) -> bool {
        let length = 18 /* size + ACK + space * 2 + continue + \n */ + self.common.len();
        let msg = format!("{:04x}ACK {} continue\n", length, self.common);
        print!("Send: {}", msg);
        self.channel.lock().unwrap().send(msg.as_bytes().to_vec()).is_ok()
    }

    fn ack_first(&self) -> bool {
        let length = 9 /* size + ACK + space + \n */ + self.common.len();
        let msg = format!("{:04x}ACK {}\n", length, self.common);
        print!("Send: {}", msg);
        self.channel.lock().unwrap().send(msg.as_bytes().to_vec()).is_ok()
    }

    fn send_pack_data(&self) {
        println!("Send: [PACKFILE]");
        // Note: this part of the code adds every commits into
        // the packfile until a commit announced by the client
        // is found AND a common parent is found (or until the
        // initial commit).
        let mut pb = self.repository.packbuilder().unwrap();
        let fetched = Oid::from_str(&*self.wanted).unwrap();
        let mut revwalk = self.repository.revwalk().unwrap();
        let _ = revwalk.push(fetched);
        let _ = revwalk.set_sorting(Sort::TOPOLOGICAL);

        let mut parents : Vec<String> = Vec::new();
        let mut have = false;

        while let Some(oid) = revwalk.next() {
            let oid = oid.unwrap();
            let oid_str = oid.to_string();
            have |= self.have.iter().find(|&o| *o == oid_str).is_some();
            if let Some(pos) = parents.iter().position(|p| *p == oid_str) {
                parents.remove(pos);
            }
            if have && parents.is_empty() {
                // All commits are fetched
                break;
            }
            let _ = pb.insert_commit(oid);
            let commit = self.repository.find_commit(oid).unwrap();
            let mut commit_parents = commit.parents();
            // Make sure to explore every branches.
            while let Some(p) = commit_parents.next() {
                parents.push(p.id().to_string());
            }
        }

        // Note: the buf can be huge. Packbuilder has some methods to get chunks.
        let mut data = Buf::new();
        let _ = pb.write_buf(&mut data);

        let len = data.len();
        let data : Vec<u8> = data.to_vec();
        let mut sent = 0;
        while sent < len {
            // cf https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt#L166
            // In 'side-band-64k' mode it will send up to 65519 data bytes plus 1 control code, for a
            // total of up to 65520 bytes in a pkt-line.
            let pkt_size = min(65515, len - sent);
            // The packet is Size (4 bytes), Control byte (0x01 for data), pack data.
            let pkt = format!("{:04x}", pkt_size + 5 /* size + control */);
            self.channel.lock().unwrap().send(pkt.as_bytes().to_vec()).unwrap();
            self.channel.lock().unwrap().send(b"\x01".to_vec()).unwrap();
            self.channel.lock().unwrap().send(data[sent..(sent+pkt_size)].to_vec()).unwrap();
            sent += pkt_size;
        }

        println!("Send: {}", FLUSH_PKT);
        // And finish by a little FLUSH
        self.channel.lock().unwrap().send(FLUSH_PKT.as_bytes().to_vec()).unwrap();
    }
}