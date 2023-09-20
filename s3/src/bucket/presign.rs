use crate::bucket::{validate_expiry, Bucket, Request};
use crate::command::Command;
use crate::error::S3Error;
use crate::post_policy::PresignedPost;
use crate::request::RequestImpl;
use crate::PostPolicy;
use http::header::HeaderMap;
use std::collections::HashMap;

impl Bucket {
    /// Get a presigned url for getting object on a given path
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse().unwrap();
    /// let credentials = Credentials::default().unwrap();
    /// let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    ///
    /// // Add optional custom queries
    /// let mut custom_queries = HashMap::new();
    /// custom_queries.insert(
    ///    "response-content-disposition".into(),
    ///    "attachment; filename=\"test.png\"".into(),
    /// );
    ///
    /// let url = bucket.presign_get("/test.file", 86400, Some(custom_queries)).unwrap();
    /// println!("Presigned url: {}", url);
    /// ```
    pub fn presign_get<S: AsRef<str>>(
        &self,
        path: S,
        expiry_secs: u32,
        custom_queries: Option<HashMap<String, String>>,
    ) -> Result<String, S3Error> {
        validate_expiry(expiry_secs)?;
        let request = RequestImpl::new(
            self,
            path.as_ref(),
            Command::PresignGet {
                expiry_secs,
                custom_queries,
            },
        )?;
        request.presigned()
    }

    /// Get a presigned url for posting an object to a given path
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use s3::post_policy::*;
    /// use std::borrow::Cow;
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse().unwrap();
    /// let credentials = Credentials::default().unwrap();
    /// let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    ///
    /// let post_policy = PostPolicy::new(86400).condition(
    ///     PostPolicyField::Key,
    ///     PostPolicyValue::StartsWith(Cow::from("user/user1/"))
    /// ).unwrap();
    ///
    /// let presigned_post = bucket.presign_post(post_policy).unwrap();
    /// println!("Presigned url: {}, fields: {:?}", presigned_post.url, presigned_post.fields);
    /// ```
    pub fn presign_post(&self, post_policy: PostPolicy) -> Result<PresignedPost, S3Error> {
        post_policy.sign(self.clone())
    }

    /// Get a presigned url for putting object to a given path
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use http::HeaderMap;
    /// use http::header::HeaderName;
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse().unwrap();
    /// let credentials = Credentials::default().unwrap();
    /// let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    ///
    /// // Add optional custom headers
    /// let mut custom_headers = HeaderMap::new();
    /// custom_headers.insert(
    ///    HeaderName::from_static("custom_header"),
    ///    "custom_value".parse().unwrap(),
    /// );
    ///
    /// let url = bucket.presign_put("/test.file", 86400, Some(custom_headers)).unwrap();
    /// println!("Presigned url: {}", url);
    /// ```
    pub fn presign_put<S: AsRef<str>>(
        &self,
        path: S,
        expiry_secs: u32,
        custom_headers: Option<HeaderMap>,
    ) -> Result<String, S3Error> {
        validate_expiry(expiry_secs)?;
        let request = RequestImpl::new(
            self,
            path.as_ref(),
            Command::PresignPut {
                expiry_secs,
                custom_headers,
            },
        )?;
        request.presigned()
    }

    /// Get a presigned url for deleting object on a given path
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse().unwrap();
    /// let credentials = Credentials::default().unwrap();
    /// let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    ///
    /// let url = bucket.presign_delete("/test.file", 86400).unwrap();
    /// println!("Presigned url: {}", url);
    /// ```
    pub fn presign_delete<S: AsRef<str>>(
        &self,
        path: S,
        expiry_secs: u32,
    ) -> Result<String, S3Error> {
        validate_expiry(expiry_secs)?;
        let request =
            RequestImpl::new(self, path.as_ref(), Command::PresignDelete { expiry_secs })?;
        request.presigned()
    }
}
