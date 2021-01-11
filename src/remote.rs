use core::fmt::Debug;
use core::{mem, result};
use core::convert::TryInto;
use std::net::{ToSocketAddrs, SocketAddr, UdpSocket};
use std::sync::mpsc::{channel, TryRecvError, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use log::{Level, debug, warn, log_enabled};
use image::RgbImage;

use crate::images::load_image;
use crate::utils::Result;

const MAX_PACKET_SIZE: usize = 4096;
const MAX_NAME_LENGTH: usize = MAX_PACKET_SIZE - 64;
const CODE_DISPLAY: u8 = b'd';
const CODE_ACK:     u8 = b'a';
const CODE_OK:      u8 = b'o';
const CODE_ERR:     u8 = b'e';

struct Timer {
    start: Instant,
    timer: Instant
}

impl Timer {
    fn new() -> Self {
        let start = Instant::now();
        Timer { start, timer: start }
    }

    fn reset(&mut self) {
        self.start = self.timer;
    }

    fn wait_if_too_fast(&mut self, min_loop_duration: Duration) {
        let elapsed = self.timer.elapsed();
        if elapsed < min_loop_duration {
            thread::sleep(min_loop_duration - elapsed);
        }
        self.timer += min_loop_duration;
    }
}

pub fn send<A: ToSocketAddrs + Debug, B: ToSocketAddrs>(
        remote: A,
        local: B,
        timeout: Duration,
        color: u32,
        name: &str
    ) -> Result<Option<bool>>
{
    if timeout.as_secs() == 0 {
        return Ok(None)
    }

    const MIN_LOOP_DURATION: Duration = Duration::from_millis(250);
    let socket = UdpSocket::bind(local)?;
    if log_enabled!(Level::Debug) {
        debug!("local {:?}", socket.local_addr()?);
        for addr in remote.to_socket_addrs()? {
            debug!("remote {:?}", addr);
        }
    }
    socket.connect(remote)?;
    socket.set_read_timeout(Some(MIN_LOOP_DURATION))?;
    let msg = RivPacket::new(color, name)?.into_inner();
    let mut buf = [0; MAX_PACKET_SIZE];
    let mut timer = Timer::new();

    while timer.timer.duration_since(timer.start) < timeout {
        let _ = socket.send(&msg);
        let packet = match socket.recv(&mut buf) {
            Ok(amt) => match RivPacket::from(&buf[0..amt]) {
                Ok(packet) if !packet.is_display() &&
                              packet.color() == color &&
                              packet.name() == name => packet,
                _ => {
                    debug!("recv invalid response");
                    continue
                }
            }
            _ => {
                timer.wait_if_too_fast(MIN_LOOP_DURATION);
                continue
            }
        };
        if packet.is_ack() {
            debug!("recv ack");
            timer.wait_if_too_fast(MIN_LOOP_DURATION);
            timer.reset();
        }
        else {
            debug!("recv resp {}", packet.is_ok());
            return Ok(Some(packet.is_ok()))
        }
    }
    Ok(None)
}

pub fn bind<A: ToSocketAddrs>(
        address: A,
        buf_width: u32,
        buf_height: u32,
        with_info: bool
    ) -> std::io::Result<Receiver<(u32, Option<RgbImage>)>>
{
    let (main_send, main_recv) = channel();
    let (work_send, work_recv) = channel();
    let (netw_send, netw_recv): (Sender<(RivPacket, SocketAddr)>, _) = channel();

    let socket = UdpSocket::bind(address)?;
    socket.set_read_timeout(Some(Duration::from_millis(50)))?;
    socket.set_write_timeout(Some(Duration::from_millis(250)))?;
    debug!("bind {:?}", socket.local_addr()?);

    // network service
    thread::spawn(move || {
        let mut udpbuf = [0; MAX_PACKET_SIZE];
        let mut last_color = 0;
        let mut last_name = String::new();
        let mut busy = false;
        loop {
            // check worker response
            match netw_recv.try_recv() {
                Ok((packet, addr)) => {
                    debug!("sending resp to {}", addr);
                    if packet.is_ok() {
                        last_color = packet.color();
                        last_name.clear();
                        last_name.push_str(packet.name());
                    }
                    let _ = socket.send_to(&packet.into_inner(), addr);
                    busy = false;
                }
                Err(TryRecvError::Disconnected) => break,
                _ => {}
            }
            // check remote request
            let (amt, src) = match socket.recv_from(&mut udpbuf) {
                Ok(msg) => msg,
                Err(..) => continue
            };
            // validate packet
            let packet = match RivPacket::from(&udpbuf[0..amt]) {
                Ok(pkt) if pkt.is_display() => pkt,
                Ok(..) => {
                    debug!("not a display packet, ignoring");
                    continue;                    
                }
                Err(err) => {
                    debug!("invalid packet, ignoring: {}", err);
                    continue;
                }
            };
            // ACK request
            udpbuf[RIVOFFS_CODE] = CODE_ACK;
            // accept request if not busy
            if !busy {
                if packet.color() == last_color && packet.name() == last_name {
                    debug!("dupe: #{:06x} {}", packet.color(), packet.name());
                    // respond immediately on dup
                    udpbuf[RIVOFFS_CODE] = CODE_OK;
                }
                else {
                    debug!("accepted: #{:06x} {}", packet.color(), packet.name());
                    // start work
                    if work_send.send((packet, src)).is_err() {
                        break;
                    }
                    busy = true;
                }
            }
            // send back ACK or OK
            let _ = socket.send_to(&udpbuf[0..amt], src);
        }
    });

    // image loader
    let load_for = move |mut packet: RivPacket, addr| -> Result<()> {
        let name = packet.name();
        if name.is_empty() {
            main_send.send((packet.color(), None))?;
            packet.set_code(CODE_OK);
        }
        else {
            debug!("loading: {}", name);
            match load_image(&name, buf_width, buf_height, with_info) {
                Ok(img) => {
                    // send to main to show it
                    main_send.send((packet.color(), Some(img)))?;
                    packet.set_code(CODE_OK);
                }
                Err(err) => {
                    warn!("loading image failed: {}", err);
                    packet.set_code(CODE_ERR);
                }
            }
        }
        // respond to network service
        netw_send.send((packet, addr))?;
        Ok(())
    };

    // image load worker
    thread::spawn(move || {
        for (packet, addr) in work_recv.iter() {
            if load_for(packet, addr).is_err() {
                break
            }
        }
    });

    Ok(main_recv)
}

/*
RIVd - picture to display
RIVa - ack
RIVo - picture shown
RIVe - error

"RIV", "d"|"a"|"o"|"e", color BE u32, filename size BE u16, filename
*/
const RIVOFFS_CODE: usize = 3;
const RIVOFFS_COLOR: usize = 4;
const RIVOFFS_COLOR_END: usize = RIVOFFS_COLOR + mem::size_of::<u32>();
const RIVOFFS_NAMELEN: usize = RIVOFFS_COLOR_END;
const RIVOFFS_NAME: usize = RIVOFFS_NAMELEN + mem::size_of::<u16>();

pub struct RivPacket {
    data: Vec<u8>
}

impl RivPacket {
    pub fn new(color: u32, name: &str) -> result::Result<Self, &str> {
        if name.len() > MAX_NAME_LENGTH {
            return Err("name is too long to encode in a packet");
        }
        let mut data = Vec::with_capacity(RIVOFFS_NAME + name.len());
        data.extend_from_slice(b"RIVd");
        data.extend_from_slice(&color.to_be_bytes());
        data.extend_from_slice(&(name.len() as u16).to_be_bytes());
        data.extend_from_slice(name.as_bytes());
        Ok(RivPacket { data })
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn from(data: &[u8]) -> result::Result<Self, &str> {
        if data.len() < RIVOFFS_NAME {
            return Err("packet too short");
        }
        let name_size = u16::from_be_bytes(data[RIVOFFS_NAMELEN..RIVOFFS_NAME].try_into().unwrap());
        if data.len() - RIVOFFS_NAME < name_size as usize {
            return Err("wrong packet file name size");
        }
        match &data[0..4] {
            b"RIVd"|b"RIVa"|b"RIVo"|b"RIVe" => {}
            _ => return Err("invalid packet magick")
        }
        core::str::from_utf8(&data[RIVOFFS_NAME..]).map_err(|_| "can't decode UTF-8")?;
        Ok(RivPacket { data: data.to_vec() })
    }

    pub fn name(&self) -> &str {
        // we did check at init
        unsafe { core::str::from_utf8_unchecked(&self.data[RIVOFFS_NAME..]) }
    }

    pub fn color(&self) -> u32 {
        u32::from_be_bytes(self.data[RIVOFFS_COLOR..RIVOFFS_COLOR_END].try_into().unwrap())
    }

    pub fn set_code(&mut self, code: u8) {
        self.data[RIVOFFS_CODE] = code;
    }

    pub fn code(&self) -> u8 {
        self.data[RIVOFFS_CODE]
    }

    pub fn is_display(&self) -> bool {
        self.code() == CODE_DISPLAY
    }

    pub fn is_ack(&self) -> bool {
        self.code() == CODE_ACK
    }

    pub fn is_ok(&self) -> bool {
        self.code() == CODE_OK
    }
}
