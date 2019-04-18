use io;

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
    pub fn on_packet<'a>(&mut self,  
                            nic: &mut tun_tap::Iface,
                            iph: etherparse::Ipv4HeaderSlice<'a>, 
                            tcph: etherparse::TcpHeaderSlice<'a>, 
                            data: &'a [u8]) 
                            -> io::Result<(usize)> {
        
        let mut buf = [0u8; 1500];

        // TODO - if we remove this print, we should put this info in the error prints in the match
        eprintln!("{}:{} -> {}:{} {}b of tcp", iph.source_addr(), tcph.source_port(), iph.destination_addr(), tcph.destination_port(), data.len());
        eprintln!("We are in state {:?}", *self);
        match *self {
            State::Closed => {
                eprintln!("We are in closed state; received unexpected packet");
                return;
            },
            State::Listen => {
                if !tcph.syn() {
                    // only expected syn packet; got sometihng else
                    eprintln!("We are in Listen state; received unexpected non-syn packet");
                    return;
                }

                // establish a connection
                let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), tcph.source_port(),unimplemented!(), unimplemented!());
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(syn_ack.slice().len(), 64, etherparse::IpTrafficClass::Tcp, iph.destination_addr(), iph.source_addr());

                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(unwritten);
                    syn_ack.write(unwritten);
                    unwritten.len()
                };
                nic.send(&buf[..unwritten])?
            },
            State::SynRcvd => {},
            State::Estab => {},
        }
    }
}
