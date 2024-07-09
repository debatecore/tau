use std::net::{Ipv4Addr, SocketAddrV4};

fn get_env_port() -> u16 {
    let portstr = match std::env::var("PORT") {
        Ok(value) => value,
        Err(_) => "2023".to_owned(),
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
