//Unlike my C solution, this one has DNS resolution.
//`cargo run www.pwnable.kr 9007`

/*
	---------------------------------------------------
	-              Shall we play a game?              -
	---------------------------------------------------
	
	You have given some gold coins in your hand
	however, there is one counterfeit coin among them
	counterfeit coin looks exactly same as real coin
	however, its weight is different from real one
	real coin weighs 10, counterfeit coin weighes 9
	help me to find the counterfeit coin with a scale
	if you find 100 counterfeit coins, you will get reward :)
	FYI, you have 60 seconds.
	
	- How to play - 
	1. you get a number of coins (N) and number of chances (C)
	2. then you specify a set of index numbers of coins to be weighed
	3. you get the weight information
	4. 2~3 repeats C time, then you give the answer
	
	- Example -
	[Server] N=4 C=2 	# find counterfeit among 4 coins with 2 trial
	[Client] 0 1 		# weigh first and second coin
	[Server] 20			# scale result : 20
	[Client] 3			# weigh fourth coin
	[Server] 10			# scale result : 10
	[Client] 2 			# counterfeit coin is third!
	[Server] Correct!

*/

use std::io::prelude::*;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::io::Read;
use std::str;
use std::env;
use std::fmt;

enum CheckState {
    CounterfeitInside,
    CounterfeitOutside,
    CounterfeitFound,
}

const READ_BYTES: usize = 2048;

#[derive(Debug, Clone)]
struct FindNumError;

impl fmt::Display for FindNumError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "number was not found")
	}
}

trait FindNum {
    fn find_num_slice(&self, start: usize) -> Result<(&[u8], usize), FindNumError>; //finds a slice containing a number
}

impl FindNum for [u8] {
    fn find_num_slice(&self, start: usize) -> Result<(&[u8], usize), FindNumError> {
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
        return Ok((&self[idx1..idx2], idx2));
    }
}

fn main() -> () {
    
    let args: Vec<String> = env::args().collect();

    //let serv_addr: String = args[1].clone();
    let serv_addr = &*args[1];
    let serv_port: u16 = args[2].parse().expect("ERROR:invalid port");

    let mut socket = TcpStream::connect(
        (serv_addr, serv_port)
        .to_socket_addrs().unwrap() //This is where the address is resolved
        .next()
        .expect("ERROR:address could not be resolved")
    ).expect("ERROR:could not esablish connection");

	let mut read_buf: [u8; READ_BYTES] = [0; READ_BYTES];
	let mut num_bytes: usize;
    
    socket.read(&mut read_buf).unwrap(); //initial mesg
	
    loop {
        println!("DEBUG:starting outer loop");

		loop{
			let read_result = socket.read(&mut read_buf);
			match read_result {
				Ok(n) => num_bytes = n,
				Err(e) => match e.kind(){
					std::io::ErrorKind::Interrupted => continue, //Retry the read()
					_ => panic!("ERROR:failed to read packet:{}", e),
				}
			}
			break; //This loop is kind of gross. I wonder if there is a more elegant way of doing this.
		}
        if num_bytes == 0 {
            break;
        }
        let read_mesg = &read_buf[0..num_bytes]; //slice of the buffer containing what was just read.
        print!("SERVER:{}", str::from_utf8(read_mesg).unwrap());

        let (n_slice, _) = read_mesg.find_num_slice(0).expect("ERROR:couldn't find number in buffer");
        //let (c_slice, _) = read_mesg.find_num_slice(end);

        let n: i64 = str::from_utf8(n_slice).unwrap().parse::<i64>().unwrap(); //Turbofish is redundant here because n has an explicit type.
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
        panic!("DEBUG:failed; time expired");
    }

    let weight = str::from_utf8(read_mesg).unwrap().parse::<i64>().unwrap();
    if weight % 2 == 1 {
        return CheckState::CounterfeitInside
    } else {
        return CheckState::CounterfeitOutside
    }
}