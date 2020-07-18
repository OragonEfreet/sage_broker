use nanoid::nanoid;
#[derive(Debug)]
pub struct Client {
    id: String,
}

impl Client {
    pub fn new() -> Client {
        Client {
            id: format!("sage_mqtt-{}", nanoid!()),
        }
    }
}
