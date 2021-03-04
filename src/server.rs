
use std::collections::HashMap;

pub struct Server {
    path: String,
    // socket: Socket, // TODO channel? tcp socket? other?
    wantedReference: String,
    common: String,
    haveReferences: Vec<String>,
    buf: Vec<u8>,
}

impl Server {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            wantedReference: String::new(),
            common: String::new(),
            haveReferences: Vec::new(),
            buf: Vec::new(),
        }
    }

    pub fn onRecv(&self, buf: Vec<u8>) {
        let mut buf = Some(buf);
        let mut needMoreParsing = true;
        while needMoreParsing {
            needMoreParsing = self.parseOrder(buf.take());
        }
    }

    fn parseOrder(&self, buf: Option<Vec<u8>>) -> bool {
        // Parse pkt len
        // Reference: https://github.com/git/git/blob/master/Documentation/technical/protocol-common.txt#L51
        // The first four bytes define the length of the packet and 0000 is a FLUSH pkt


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
        true
    }

    fn ACKCommon(&self) -> bool {
        true
    }

    fn ACKFirst(&self) -> bool {
        true
    }

    fn sendPackData(&self) {

    }

    fn params(pkt_line: &String) -> HashMap<String, String> {
        HashMap::new()
    }
}