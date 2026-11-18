pub const VERSION: u8 = 1;
pub const OP_REGISTER: u8 = 1;
pub const OP_SET: u8 = 2;
pub const OP_INVOKE: u8 = 3;
pub const STATUS_OK: u8 = 0;
pub const STATUS_ERR: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register { code: Vec<u8> },
    Set { id: Vec<u8>, code: Vec<u8> },
    Invoke { id: Vec<u8>, input: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u8,
    pub body: Vec<u8>,
}
