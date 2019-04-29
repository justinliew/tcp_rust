use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;

extern crate tun_tap;
extern crate etherparse;

mod tcp;

#[derive(Debug,Clone,Copy,Hash,Eq,PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16)
}

fn main() -> io::Result<()> {
    let mut connections : HashMap<Quad, tcp::Connection> = Default::default();
    
    let mut nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                if iph.protocol() != 0x06 {
                    eprintln!("Ignoring non-tcp packet");
                    // ignore non-tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        use std::collections::hash_map::Entry;
                        let datai = iph.slice().len() + tcph.slice().len();
                        match connections.entry(Quad{
                            src: (src, tcph.source_port()),
                            dst: (dst, tcph.destination_port()),
                        }) {
                            Entry::Occupied(mut c) => {
                                c.get_mut().on_packet(&mut nic, iph, tcph, &buf[datai..nbytes])?
                            }
                            Entry::Vacant(mut e) => {
                                if let Some(c) = tcp::Connection::accept(&mut nic, iph, tcph, &buf[datai..nbytes])? {
                                    e.insert(c);
                                }
                            }

                        }
                    },
                    Err(e) => {
                        eprintln!("ignoring weird tcp packet {:?}",e);
                    }
                }
            },
            Err(e) => {
                // eprintln!("ignoring weird packet {:?}", e);
            }
        }

    }
}
