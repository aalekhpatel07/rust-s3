use crate::bucket::Bucket;
use crate::command::Command;
use crate::error::S3Error;
use crate::request::Request;
use crate::request::RequestImpl;
use crate::serde_types::{ListBucketResult, ListMultipartUploadsResult};
use awscreds::Credentials;
use awsregion::Region;
use serde::Deserialize;

impl Bucket {
    /// Get a list of all existing buckets in the region
    /// that are accessible by the given credentials.
    /// ```no_run
    /// use s3::{Bucket, BucketConfiguration};
    /// use s3::creds::Credentials;
    /// use s3::region::Region;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let region = Region::Custom {
    ///   region: "eu-central-1".to_owned(),
    ///   endpoint: "http://localhost:9000".to_owned()
    /// };
    /// let credentials = Credentials::default()?;
    ///
    /// let response = Bucket::list_buckets(region, credentials).await?;
    ///
    /// let found_buckets = response.bucket_names().collect::<Vec<String>>();
    /// println!("found buckets: {:#?}", found_buckets);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_buckets(
        region: Region,
        credentials: Credentials,
    ) -> Result<crate::bucket::ListBucketsResponse, S3Error> {
        let dummy_bucket = Bucket::new("", region, credentials)?.with_path_style();
        let request = RequestImpl::new(&dummy_bucket, "", Command::ListBuckets)?;
        let response = request.response_data(false).await?;

        Ok(quick_xml::de::from_str::<crate::bucket::ListBucketsResponse>(response.as_str()?)?)
    }

    /// Determine whether the instantiated bucket exists.
    /// ```no_run
    /// use s3::{Bucket, BucketConfiguration};
    /// use s3::creds::Credentials;
    /// use s3::region::Region;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let bucket_name = "some-bucket-that-is-known-to-exist";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    ///
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let exists = bucket.exists().await?;
    ///
    /// assert_eq!(exists, true);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn exists(&self) -> Result<bool, S3Error> {
        let credentials = self
            .credentials
            .read()
            .expect("Read lock to be acquired on Credentials")
            .clone();

        let response = Self::list_buckets(self.region.clone(), credentials).await?;

        Ok(response
            .bucket_names()
            .collect::<std::collections::HashSet<String>>()
            .contains(&self.name))
    }

    pub async fn list_page(
        &self,
        prefix: String,
        delimiter: Option<String>,
        continuation_token: Option<String>,
        start_after: Option<String>,
        max_keys: Option<usize>,
    ) -> Result<(ListBucketResult, u16), S3Error> {
        let command = if self.listobjects_v2 {
            Command::ListObjectsV2 {
                prefix,
                delimiter,
                continuation_token,
                start_after,
                max_keys,
            }
        } else {
            // In the v1 ListObjects request, there is only one "marker"
            // field that serves as both the initial starting position,
            // and as the continuation token.
            Command::ListObjects {
                prefix,
                delimiter,
                marker: std::cmp::max(continuation_token, start_after),
                max_keys,
            }
        };
        let request = RequestImpl::new(self, "/", command)?;
        let response_data = request.response_data(false).await?;
        let list_bucket_result = quick_xml::de::from_reader(response_data.as_slice())?;

        Ok((list_bucket_result, response_data.status_code()))
    }

    /// List the contents of an S3 bucket.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let results = bucket.list("/".to_string(), Some("/".to_string())).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(
        &self,
        prefix: String,
        delimiter: Option<String>,
    ) -> Result<Vec<ListBucketResult>, S3Error> {
        let the_bucket = self.to_owned();
        let mut results = Vec::new();
        let mut continuation_token = None;

        loop {
            let (list_bucket_result, _) = the_bucket
                .list_page(
                    prefix.clone(),
                    delimiter.clone(),
                    continuation_token,
                    None,
                    None,
                )
                .await?;
            continuation_token = list_bucket_result.next_continuation_token.clone();
            results.push(list_bucket_result);
            if continuation_token.is_none() {
                break;
            }
        }

        Ok(results)
    }

    pub async fn list_multiparts_uploads_page(
        &self,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        key_marker: Option<String>,
        max_uploads: Option<usize>,
    ) -> Result<(ListMultipartUploadsResult, u16), S3Error> {
        let command = Command::ListMultipartUploads {
            prefix,
            delimiter,
            key_marker,
            max_uploads,
        };
        let request = RequestImpl::new(self, "/", command)?;
        let response_data = request.response_data(false).await?;
        let list_bucket_result = quick_xml::de::from_reader(response_data.as_slice())?;

        Ok((list_bucket_result, response_data.status_code()))
    }

    /// List the ongoing multipart uploads of an S3 bucket. This may be useful to cleanup failed
    /// uploads, together with [`crate::bucket::Bucket::abort_upload`].
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let results = bucket.list_multiparts_uploads(Some("/"), Some("/")).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_multiparts_uploads(
        &self,
        prefix: Option<&str>,
        delimiter: Option<&str>,
    ) -> Result<Vec<ListMultipartUploadsResult>, S3Error> {
        let the_bucket = self.to_owned();
        let mut results = Vec::new();
        let mut next_marker: Option<String> = None;

        loop {
            let (list_multiparts_uploads_result, _) = the_bucket
                .list_multiparts_uploads_page(prefix, delimiter, next_marker, None)
                .await?;

            let is_truncated = list_multiparts_uploads_result.is_truncated;
            next_marker = list_multiparts_uploads_result.next_marker.clone();
            results.push(list_multiparts_uploads_result);

            if !is_truncated {
                break;
            }
        }

        Ok(results)
    }
}

#[derive(Clone, Default, Deserialize, Debug)]
#[serde(rename_all = "PascalCase", rename = "ListAllMyBucketsResult")]
pub struct ListBucketsResponse {
    pub owner: BucketOwner,
    pub buckets: BucketContainer,
}

impl ListBucketsResponse {
    pub fn bucket_names(&self) -> impl Iterator<Item = String> + '_ {
        self.buckets.bucket.iter().map(|bucket| bucket.name.clone())
    }
}

#[derive(Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct BucketOwner {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: crate::serde_types::DateTime,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BucketContainer {
    #[serde(default)]
    pub bucket: Vec<BucketInfo>,
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn parse_list_buckets_response() {
        let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
            <ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
                <Owner>
                    <ID>02d6176db174dc93cb1b899f7c6078f08654445fe8cf1b6ce98d8855f66bdbf4</ID>
                    <DisplayName>minio</DisplayName>
                </Owner>
                <Buckets>
                    <Bucket>
                        <Name>test-rust-s3</Name>
                        <CreationDate>2023-06-04T20:13:37.837Z</CreationDate>
                    </Bucket>
                    <Bucket>
                        <Name>test-rust-s3-2</Name>
                        <CreationDate>2023-06-04T20:17:47.152Z</CreationDate>
                    </Bucket>
                </Buckets>
            </ListAllMyBucketsResult>
        "#;

        let parsed = quick_xml::de::from_str::<super::ListBucketsResponse>(response).unwrap();

        assert_eq!(parsed.owner.display_name, "minio");
        assert_eq!(
            parsed.owner.id,
            "02d6176db174dc93cb1b899f7c6078f08654445fe8cf1b6ce98d8855f66bdbf4"
        );
        assert_eq!(parsed.buckets.bucket.len(), 2);

        assert_eq!(parsed.buckets.bucket.first().unwrap().name, "test-rust-s3");
        assert_eq!(
            parsed.buckets.bucket.first().unwrap().creation_date,
            "2023-06-04T20:13:37.837Z"
                .parse::<crate::serde_types::DateTime>()
                .unwrap()
        );

        assert_eq!(parsed.buckets.bucket.last().unwrap().name, "test-rust-s3-2");
        assert_eq!(
            parsed.buckets.bucket.last().unwrap().creation_date,
            "2023-06-04T20:17:47.152Z"
                .parse::<crate::serde_types::DateTime>()
                .unwrap()
        );
    }

    #[test]
    pub fn parse_list_buckets_response_when_no_buckets_exist() {
        let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
            <ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
                <Owner>
                    <ID>02d6176db174dc93cb1b899f7c6078f08654445fe8cf1b6ce98d8855f66bdbf4</ID>
                    <DisplayName>minio</DisplayName>
                </Owner>
                <Buckets>
                </Buckets>
            </ListAllMyBucketsResult>
        "#;

        let parsed = quick_xml::de::from_str::<super::ListBucketsResponse>(response).unwrap();

        assert_eq!(parsed.owner.display_name, "minio");
        assert_eq!(
            parsed.owner.id,
            "02d6176db174dc93cb1b899f7c6078f08654445fe8cf1b6ce98d8855f66bdbf4"
        );
        assert_eq!(parsed.buckets.bucket.len(), 0);
    }
}
