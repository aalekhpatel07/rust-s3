use crate::{bucket::Bucket, error::S3Error};


impl Bucket {
    pub fn credentials_refresh(&self) -> Result<(), S3Error> {
        Ok(self
            .credentials
            .try_write()
            .map_err(|_| S3Error::WLCredentials)?
            .refresh()?)
    }
}