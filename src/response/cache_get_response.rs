#[derive(Debug)]
pub enum MomentoGetStatus {
    HIT,
    MISS,
    ERROR
}

#[derive(Debug)]
pub struct MomentoGetResponse {
    pub result: MomentoGetStatus,
    pub value: Vec<u8>
}