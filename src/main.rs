
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::io::Read;
use std::str;
use std::error::Error;
use std::env;

enum CheckState {
    CounterfeitInside,
    CounterfeitOutside,
    CounterfeitFound,
}

const READ_BYTES: usize = 2048;

trait FindNum {
    fn find_num_slice(&self, start: usize) -> (&[u8], usize); //finds a slice containing a number
}

impl FindNum for [u8] {
    fn find_num_slice(&self, start: usize) -> (&[u8], usize) {
        //NOTE: currently does not handle the possibility of a number not being found.
        const NUMS_STR: &[u8] = b"1234567890";
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
            if n == false {
                break;
            }
            idx2 += 1;
        }
        //println!("{:?}", &self[idx1..idx2]);
        return (&self[idx1..idx2], idx2);
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
        .expect("ERROR:Address could not be resolved.")
    )?;

    let mut read_buf: [u8; READ_BYTES] = [0; READ_BYTES];
    let mut num_bytes: usize;
    
    socket.read(&mut read_buf)?; //initial mesg

    loop {
        println!("DEBUG:starting outer loop");
        num_bytes = socket.read(&mut read_buf)?;
        if num_bytes == 0 {
            break;
        }
        let read_mesg = &read_buf[0..num_bytes]; //slice of the buffer containing what was just read.
        print!("SERVER:{}", str::from_utf8(read_mesg)?);

        let (n_slice, _) = read_mesg.find_num_slice(0);
        //let (c_slice, _) = read_mesg.find_num_slice(end);

        let n: i64 = str::from_utf8(n_slice)?.parse::<i64>()?;
        println!("DEBUG:n = {}", n);
        //let c: i64 = str::from_utf8(c_slice)?.parse::<i64>()?; //Doesn't get used...

        let mut range_beg = 0;
        let mut range_end = n - 1;
        let mut range_mid = (range_end + range_beg) / 2;

        'out: loop {
            let state = check(range_beg, range_mid, &mut read_buf, &mut socket);
            match state {
                CheckState::CounterfeitOutside => {
                range_beg = range_mid+1;
            },
                CheckState::CounterfeitInside => {
                range_end = range_mid
            }
                CheckState::CounterfeitFound => {
                    break 'out
                }
        }
        range_mid = (range_end + range_beg) / 2;
        }
    }
    Ok(())
}

fn check(range_beg: i64, range_end: i64, read_buf: &mut [u8], socket: &mut TcpStream) -> CheckState {
    let mut write_mesg: Vec<u8> = Vec::new();
    
    for i in range_beg..range_end+1 {
        write_mesg.extend_from_slice(&*i.to_string().as_bytes());
        write_mesg.push(' ' as u8);
    }
    let write_mesg_len = write_mesg.len();
    write_mesg[write_mesg_len-1] = '\n' as u8; //Change the last character to newline.
    print!("CLIENT:{}", unsafe{ str::from_utf8_unchecked(&*write_mesg) });
    socket.write(&*write_mesg).unwrap();

    let num_bytes = socket.read(read_buf).unwrap();
    let read_mesg = &read_buf[0..num_bytes-1];
    println!("SERVER:{}", str::from_utf8(read_mesg).unwrap());

    if read_mesg[0] == 'C' as u8{
        return CheckState::CounterfeitFound
    }
    if read_mesg[0] == 't' as u8 {
        panic!("DEBUG:Failed; time expired");
    }

    let weight = str::from_utf8(read_mesg).unwrap().parse::<i64>().unwrap();
    if weight % 2 == 1 {
        return CheckState::CounterfeitInside
    } else {
        return CheckState::CounterfeitOutside
    }
}