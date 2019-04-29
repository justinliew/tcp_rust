use std::io;

#[derive(Debug)]
enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}

pub struct Connection {
    state: State,
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
        eprintln!("got ip header: {:02x?}", iph);
        eprintln!("got tcp header: {:02x?}", tcph);
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
        };

        // establish a connection
        let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), 
                                                    tcph.source_port(),
                                                    c.send.iss,  
                                                    c.send.wnd);
        syn_ack.syn = true;
        syn_ack.ack = true;
        syn_ack.acknowledgment_number = c.recv.nxt;

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
        syn_ack.checksum = syn_ack.calc_checksum_ipv4(&ip,&[]).expect("could not calculate checksum");

        let unwritten = {
            let mut unwritten = &mut buf[..];
            ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            unwritten.len()
        };
        eprintln!("responding with {:02x?}", &buf[..buf.len() - unwritten]);
        nic.send(&buf[..unwritten]);
        Ok(Some(c))
    }

    pub fn on_packet<'a>(&mut self,  
                            nic: &mut tun_tap::Iface,
                            iph: etherparse::Ipv4HeaderSlice<'a>, 
                            tcph: etherparse::TcpHeaderSlice<'a>, 
                            data: &'a [u8]) 
                            -> io::Result<()> {
                                Ok(())
                            }
}
