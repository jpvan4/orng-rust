#[derive(Debug)]
pub struct Share {
    pub job_id: String,
    pub nonce: Vec<u8>,
    pub hash: Vec<u8>,
}

impl Share {
    pub fn new(job_id: String, nonce: u32, hash: Vec<u8>) -> Self {
        Share {
            job_id,
            nonce: nonce.to_le_bytes().to_vec(),
            hash,
        }
    }
}