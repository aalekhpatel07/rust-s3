use std::pin::Pin;

pub type DataStream = Pin<Box<dyn futures::Stream<Item = StreamItem> + Send>>;
pub type StreamItem = Result<bytes::Bytes, crate::error::S3Error>;

pub struct ResponseDataStream {
    pub bytes: DataStream,
    pub status_code: u16,
}

impl ResponseDataStream {
    pub fn bytes(&mut self) -> &mut DataStream {
        &mut self.bytes
    }
}
