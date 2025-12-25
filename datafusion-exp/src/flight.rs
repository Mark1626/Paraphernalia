#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCommand {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutCommand {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
