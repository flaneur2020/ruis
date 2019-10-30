struct Client {
    addr: String,
    password: Option<String>,
    max_idle_conns: usize,
}

impl Client {
    fn new(addr: String, password: Option<String>) -> Client {
        Client {
            addr: addr,
            password: password,
            max_idle_conns: 4,
        }
    }
}
