use std::io;

#[derive(Debug)]
enum State {
    //Listen,
    SynRcvd,
    Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    ip: etherparse::Ipv4Header,
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

                   1         2          3          4
              ----------|----------|----------|----------
                     SND.UNA    SND.NXT    SND.UNA
                                          +SND.WND

        1 - old sequence numbers which have been acknowledged
        2 - sequence numbers of unacknowledged data
        3 - sequence numbers allowed for new data transmission
        4 - future sequence numbers which are not yet allowed      
*/
struct SendSequenceSpace {
    una: u32,     // send unacknowledged
    nxt: u32,     // send next
    wnd: u16,     // send window
    up: bool,       // send urgent pointer
    wl1: u32,     // segment sequence number used for last window update
    wl2: u32,     // segment acknowledgement number used for last window update 
    iss: u32,     // initial send sequence number
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
    nxt: u32,
    wnd: u16,
    up: bool,
    irs: u32,
}

impl Connection {
    pub fn accept<'a>(nic: &mut tun_tap::Iface,
                        iph: etherparse::Ipv4HeaderSlice<'a>, 
                        tcph: etherparse::TcpHeaderSlice<'a>, 
                        data: &'a [u8]) 
                        -> io::Result<Option<Self>> {
        
        let mut buf = [0u8; 1500];

        // TODO - if we remove this print, we should put this info in the error prints in the match
        eprintln!("{}:{} -> {}:{} {}b of tcp", iph.source_addr(), tcph.source_port(), iph.destination_addr(), tcph.destination_port(), data.len());
        if !tcph.syn() {
            // only expected syn packet; got sometihng else
            eprintln!("We are in Listen state; received unexpected non-syn packet");
            return Ok(None);
        }

        let iss = 0;
        let mut c = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,

                wl1: 0,
                wl2: 0, 
            },
            recv: RecvSequenceSpace {
                nxt : tcph.sequence_number() + 1,
                wnd : tcph.window_size(),
                irs : tcph.sequence_number(),
                up : false,
            },
            ip: etherparse::Ipv4Header::new(
                0,
                64, 
                etherparse::IpTrafficClass::Tcp, 
                [
                    iph.destination()[0], iph.destination()[1], iph.destination()[2], iph.destination()[3],
                ],
                [
                    iph.source()[0], iph.source()[1], iph.source()[2], iph.source()[3],
                ])
            };

        // establish a connection
        let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), 
                                                    tcph.source_port(),
                                                    c.send.iss,  
                                                    c.send.wnd);
        syn_ack.syn = true;
        syn_ack.ack = true;
        syn_ack.acknowledgment_number = c.recv.nxt;
        c.ip.set_payload_len(syn_ack.header_len() as usize);

        let unwritten = {
            let mut unwritten = &mut buf[..];
            c.ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            unwritten.len()
        };
        nic.send(&buf[..unwritten]);
        Ok(Some(c))
    }

    fn is_between_wrapped(start: u32, x: u32, end: u32) -> bool {
        use std::cmp::{Ord,Ordering};
        match start.cmp(&end) {
            Ordering::Equal => {
                false
            },
            Ordering::Less => {
                (start < x && x < end)
            },
            Ordering::Greater => {
                (x > start || x < end)
            }
        }
    }

    pub fn on_packet<'a>(
        &mut self,  
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>, 
        tcph: etherparse::TcpHeaderSlice<'a>, 
        data: &'a [u8]) 
        -> io::Result<()> {
            // acceptable ACK check: 
            // SND.UNA < SEG.ACK =< SND.NXT
            let ackn = tcph.acknowledgment_number();
            if !Connection::is_between_wrapped(self.send.una, ackn, self.send.nxt) || ackn == self.send.nxt {
                return Ok(())
            }

            // valid segment check
            // RCV.NXT =< SEG.SEQ < RCV.NXT+RCV.WND
            let seqn = tcph.sequence_number();
            if !Connection::is_between_wrapped(self.recv.nxt, seqn, self.recv.nxt + self.recv.wnd as u32) || seqn == self.recv.nxt {
                return Ok(())
            }

            // if tcph.acknowledgment_number
            match self.state {
                State::SynRcvd => {
                    // expect to get an ACK for our SYN

                }
                State::Estab => {
                    unimplemented!();
                }
            }
            Ok(())
        }
}
