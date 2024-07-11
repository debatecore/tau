use std::net::{Ipv4Addr, SocketAddrV4};

fn get_env_port() -> u16 {
    let portstr = match std::env::var("PORT") {
        Ok(value) => value,
        Err(_) => return 2023,
    };

    return match portstr.parse() {
        Ok(num) => num,
        Err(e) => {
            panic!("Could not parse env var PORT! ({e})");
        }
    };
}

pub fn get_socket_addr() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), get_env_port())
}
