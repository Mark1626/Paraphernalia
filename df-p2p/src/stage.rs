use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Stage {
    pub stage_id: usize,
    pub encoded_plan: Bytes,
}
