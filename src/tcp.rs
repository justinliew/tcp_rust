#[derive(Debug,Clone,Copy,Hash,Eq,PartialEq)]
pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}

impl Default for State {
    fn default() -> Self {
        // State::Closed // temporary for easy testing
        State::Listen
    }
}

impl State {
    pub fn on_packet<'a>(&mut self, iph: etherparse::Ipv4HeaderSlice<'a>, tcph: etherparse::TcpHeaderSlice<'a>, data: &'a [u8]) {
        eprintln!("{}:{} -> {}:{} {}b of tcp", iph.source_addr(), tcph.source_port(), iph.destination_addr(), tcph.destination_port(), data.len());
        eprintln!("We are in state {:?}", *self);
        match *self {
            State::Closed => {
                return;
            },
            State::Listen => {
                if !tcph.syn() {
                    // only expected syn packet; got sometihng else
                    return;
                }

                // establish a connection
                let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), tcph.source_port(),0,0);
                syn_ack.syn = true;
                syn_ack.ack = true;
            },
            State::SynRcvd => {},
            State::Estab => {},
        }
    }
}
