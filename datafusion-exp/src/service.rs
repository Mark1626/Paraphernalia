use std::collections::HashMap;

use arrow::{array::RecordBatch, ipc::writer::StreamWriter};
use tonic::{Request, Response, Status};

use crate::mark::{FetchRecordBatchRequest, FetchRecordBatchResponse, df_proto_server::DfProto};

pub struct ProtoServiceImpl {
    data: HashMap<String, Vec<RecordBatch>>,
}

impl ProtoServiceImpl {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, id: String, record_batch: Vec<RecordBatch>) {
        self.data.insert(id, record_batch);
    }
}

#[tonic::async_trait]
impl DfProto for ProtoServiceImpl {
    async fn fetch_batches(
        &self,
        request: Request<FetchRecordBatchRequest>,
    ) -> Result<Response<FetchRecordBatchResponse>, Status> {
        let data = request.into_inner();

        let batches = self.data.get(&data.id).unwrap();
        let schema = batches[0].schema();

        let mut stream = vec![];
        let mut writer = StreamWriter::try_new(&mut stream, &schema).unwrap();
        for batch in batches {
            writer.write(batch).unwrap();
        }
        writer.finish().unwrap();

        Ok(Response::new(FetchRecordBatchResponse { batches: stream }))
    }
}
