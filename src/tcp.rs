use std::io;

enum State {
    Closed,
    Listen,
    // SynRcvd,
    // Estab,
}

pub struct Connection {
    state: State
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}

/*
    Send Sequence Variables

      SND.UNA - send unacknowledged
      SND.NXT - send next
      SND.WND - send window
      SND.UP  - send urgent pointer
      SND.WL1 - segment sequence number used for last window update
      SND.WL2 - segment acknowledgment number used for last window
                update
      ISS     - initial send sequence number
*/
struct SendSequenceSpace {
    una: usize,     // send unacknowledged
    nxt: usize,     // send next
    wnd: usize,     // send window
    up: bool,       // send urgent pointer
    wl1: usize,     // segment sequence number used for last window update
    wl2: usize,     // segment acknowledgement number used for last window update 
    iss: usize,     // initial send sequence number
}

/*
  Receive Sequence Space

                       1          2          3
                   ----------|----------|----------
                          RCV.NXT    RCV.NXT
                                    +RCV.WND

        1 - old sequence numbers which have been acknowledged
        2 - sequence numbers allowed for new reception
        3 - future sequence numbers which are not yet allowed

                         Receive Sequence Space
*/
struct RecvSequenceSpace {
    nxt: usize,
    wnd: usize,
    up: bool,
    irs: usize,
}

impl Default for Connection {
    fn default() -> Self {
        // State::Closed // temporary for easy testing
        Connection {
            state: State::Listen,
        }
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
                return Ok(0);
            },
            State::Listen => {
                if !tcph.syn() {
                    // only expected syn packet; got sometihng else
                    eprintln!("We are in Listen state; received unexpected non-syn packet");
                    return Ok(0);
                }

                // establish a connection
                let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), 
                                                            tcph.source_port(),
                                                            unimplemented!(), 
                                                            unimplemented!());
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(syn_ack.header_len(), 
                                                            64, 
                                                            etherparse::IpTrafficClass::Tcp, 
                                                            [
                                                                iph.destination()[0],
                                                                iph.destination()[1],
                                                                iph.destination()[2],
                                                                iph.destination()[3],
                                                            ],
                                                            [
                                                                iph.source()[0],
                                                                iph.source()[1],
                                                                iph.source()[2],
                                                                iph.source()[3],
                                                            ]);

                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };
                nic.send(&buf[..unwritten])
            },
        }
    }
}
