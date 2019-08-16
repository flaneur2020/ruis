struct Client {
    addr: String,
    password: String,
    max_idle_conns: usize,
}

impl Client {
    fn new(addr: String, password: String) -> Client {
        Client {
            addr: addr,
            password: password,
            max_idle_conns: 4,
        }
    }
}