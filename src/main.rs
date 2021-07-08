
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::io::Read;
use std::str;
use std::error::Error;
use std::env;

const READ_BYTES: usize = 2048;

trait FindNum {
    fn find_num_slice(&self, start: usize) -> &[u8];
}

impl FindNum for [u8] {
    fn find_num_slice(&self, start: usize) -> &[u8] {
        const NUMS_STR: &[u8] = b"123456890";
        let mut idx1: usize = start;
        'outer: while idx1 < self.len() {
            for i in NUMS_STR {
                if self[idx1] == *i {
                    break 'outer;
                }
            }
            idx1 += 1;
        }

        let mut idx2 = idx1;
        while idx2 < self.len() {
            let mut n: bool = false;
            for i in NUMS_STR {
                if self[idx2] == *i {
                    n = true;
                }
            }
            idx2 += 1;
            if n == false {
                break;
            }
        }
        return &self[idx1..idx2];
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    
    let args: Vec<String> = env::args().collect();

    //let serv_addr: String = args[1].clone();
    let serv_addr = &*args[1];
    let serv_port: u16 = args[2].parse()?;

    let mut socket = TcpStream::connect(
        (serv_addr, serv_port)
        .to_socket_addrs()? //This is where the address is resolved
        .next()
        .expect("Address could not be resolved.")
    )?;

    let mut read_buf: [u8; READ_BYTES] = [0; READ_BYTES];
    let mut num_bytes: usize;
    
    socket.read(&mut read_buf)?; //initial mesg

    loop {
        num_bytes = socket.read(&mut read_buf)?;
        if num_bytes == 0 {
            break;
        }
        let read_mesg = &read_buf[0..num_bytes];
        print!("{}", str::from_utf8(read_mesg)?);
        
    }
    Ok(())
}
