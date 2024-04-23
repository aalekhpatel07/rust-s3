extern crate base64;
extern crate md5;

use bytes::Buf;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::BodyStream;
use http_body_util::Full;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::io::Read;
use time::OffsetDateTime;
use tokio::{io::AsyncWriteExt as _, net::TcpStream};

use super::request_trait::ResponseData;
use crate::bucket::Bucket;
use crate::command::Command;
use crate::command::HttpMethod;
use crate::error::S3Error;

pub use crate::request::tokio_backend::HyperRequest as RequestImpl;
pub use tokio_stream::Stream;

use tracing::{event, span, Level};

use crate::request::request_trait::ResponseDataStream;

// Temporary structure for making a request
pub struct HyperRequest<'a> {
    pub bucket: &'a Bucket,
    pub path: &'a str,
    pub command: Command<'a>,
    pub datetime: OffsetDateTime,
}

#[async_trait::async_trait]
impl<'a> crate::request::Request for HyperRequest<'a> {
    type Response = http::Response<hyper::body::Incoming>;
    type HeaderMap = http::header::HeaderMap;

    async fn response(&self) -> Result<http::Response<hyper::body::Incoming>, S3Error> {
        let headers = match self.headers() {
            Ok(headers) => headers,
            Err(e) => return Err(e),
        };

        let url = self.url()?;
        let host = url.host().unwrap();
        let port = url.port_u16().unwrap_or(80);
        let address = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&address).await?;

        let io = TokioIo::new(stream);
        let (mut sender, connection) = hyper::client::conn::http1::handshake(io).await?;

        // Poll the connection
        tokio::task::spawn(async move {
            if let Err(err) = connection.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let method = match self.command.http_verb() {
            HttpMethod::Delete => http::Method::DELETE,
            HttpMethod::Get => http::Method::GET,
            HttpMethod::Post => http::Method::POST,
            HttpMethod::Put => http::Method::PUT,
            HttpMethod::Head => http::Method::HEAD,
        };

        let request = {
            let authority = url.authority().unwrap().clone();

            let mut request = http::Request::builder()
                .header(hyper::header::HOST, authority.as_str())
                .method(method)
                .uri(url.path());

            for (header, value) in headers.iter() {
                request = request.header(header, value);
            }

            let bytes = Bytes::from(self.request_body());
            request.body(Full::new(bytes))?
        };

        let span = span!(
            Level::DEBUG,
            "rust-s3-async",
            bucket = self.bucket.name(),
            command = self.command.to_string(),
            path = self.path,
            second = self.datetime.second(),
            minute = self.datetime.minute(),
            hour = self.datetime.hour(),
            day = self.datetime.day(),
            month = self.datetime.month() as u8,
            year = self.datetime.year()
        );
        let _enter = span.enter();
        let response = sender.send_request(request).await?;

        event!(Level::DEBUG, status_code = response.status().as_u16(),);

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.collect().await?.aggregate();
            let mut text = String::default();
            body.reader().read_to_string(&mut text)?;
            return Err(S3Error::HttpFailWithBody(status, text));
        }

        Ok(response)
    }

    async fn response_data(&self, etag: bool) -> Result<ResponseData, S3Error> {
        let response = self.response().await?;
        let status_code = response.status().as_u16();
        let mut headers = response.headers().clone();
        let response_headers = headers
            .clone()
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    v.to_str()
                        .unwrap_or("could-not-decode-header-value")
                        .to_string(),
                )
            })
            .collect::<HashMap<String, String>>();
        let body_vec = if etag {
            if let Some(etag) = headers.remove("ETag") {
                Bytes::from(etag.to_str()?.to_string())
            } else {
                Bytes::from("")
            }
        } else {
            let reader = response.collect().await?.aggregate().reader();
            let bytes = reader.bytes().collect::<Result<Vec<_>, _>>().unwrap();
            Bytes::from(bytes)
        };
        Ok(ResponseData::new(body_vec, status_code, response_headers))
    }

    async fn response_data_to_writer<T: tokio::io::AsyncWrite + Send + Unpin>(
        &self,
        writer: &mut T,
    ) -> Result<u16, S3Error> {
        let response = self.response().await?;

        let status_code = response.status();
        let mut stream = response.into_body();

        while let Some(item) = stream.frame().await {
            let item = item?;
            if let Some(chunk) = item.data_ref() {
                writer.write_all(chunk).await?;
            }
        }

        Ok(status_code.as_u16())
    }

    async fn response_data_to_stream(&self) -> Result<ResponseDataStream, S3Error> {
        let response = self.response().await?;
        let status_code = response.status();
        let body_stream = BodyStream::new(response);
        Ok(ResponseDataStream {
            bytes: Box::pin(body_stream),
            status_code: status_code.as_u16(),
        })
    }

    async fn response_header(&self) -> Result<(Self::HeaderMap, u16), S3Error> {
        let response = self.response().await?;
        let status_code = response.status().as_u16();
        let headers = response.headers().clone();
        Ok((headers, status_code))
    }

    fn datetime(&self) -> OffsetDateTime {
        self.datetime
    }

    fn bucket(&self) -> Bucket {
        self.bucket.clone()
    }

    fn command(&self) -> Command {
        self.command.clone()
    }

    fn path(&self) -> String {
        self.path.to_string()
    }
}

impl<'a> HyperRequest<'a> {
    pub fn new(
        bucket: &'a Bucket,
        path: &'a str,
        command: Command<'a>,
    ) -> Result<HyperRequest<'a>, S3Error> {
        bucket.credentials_refresh()?;
        Ok(Self {
            bucket,
            path,
            command,
            datetime: OffsetDateTime::now_utc(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::bucket::Bucket;
    use crate::command::Command;
    use crate::request::tokio_backend::HyperRequest;
    use crate::request::Request;
    use awscreds::Credentials;
    use http::header::{HOST, RANGE};

    // Fake keys - otherwise using Credentials::default will use actual user
    // credentials if they exist.
    fn fake_credentials() -> Credentials {
        let access_key = "AKIAIOSFODNN7EXAMPLE";
        let secert_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        Credentials::new(Some(access_key), Some(secert_key), None, None, None).unwrap()
    }

    #[test]
    fn url_uses_https_by_default() {
        let region = "custom-region".parse().unwrap();
        let bucket = Bucket::new("my-first-bucket", region, fake_credentials()).unwrap();
        let path = "/my-first/path";
        let request = HyperRequest::new(&bucket, path, Command::GetObject).unwrap();

        assert_eq!(request.url().unwrap().scheme_str(), Some("https"));

        let headers = request.headers().unwrap();
        let host = headers.get(HOST).unwrap();

        assert_eq!(*host, "my-first-bucket.custom-region".to_string());
    }

    #[test]
    fn url_uses_https_by_default_path_style() {
        let region = "custom-region".parse().unwrap();
        let bucket = Bucket::new("my-first-bucket", region, fake_credentials())
            .unwrap()
            .with_path_style();
        let path = "/my-first/path";
        let request = HyperRequest::new(&bucket, path, Command::GetObject).unwrap();

        assert_eq!(request.url().unwrap().scheme_str(), Some("https"));

        let headers = request.headers().unwrap();
        let host = headers.get(HOST).unwrap();

        assert_eq!(*host, "custom-region".to_string());
    }

    #[test]
    fn url_uses_scheme_from_custom_region_if_defined() {
        let region = "http://custom-region".parse().unwrap();
        let bucket = Bucket::new("my-second-bucket", region, fake_credentials()).unwrap();
        let path = "/my-second/path";
        let request = HyperRequest::new(&bucket, path, Command::GetObject).unwrap();

        assert_eq!(request.url().unwrap().scheme_str(), Some("http"));

        let headers = request.headers().unwrap();
        let host = headers.get(HOST).unwrap();
        assert_eq!(*host, "my-second-bucket.custom-region".to_string());
    }

    #[test]
    fn url_uses_scheme_from_custom_region_if_defined_with_path_style() {
        let region = "http://custom-region".parse().unwrap();
        let bucket = Bucket::new("my-second-bucket", region, fake_credentials())
            .unwrap()
            .with_path_style();
        let path = "/my-second/path";
        let request = HyperRequest::new(&bucket, path, Command::GetObject).unwrap();

        assert_eq!(request.url().unwrap().scheme_str(), Some("http"));

        let headers = request.headers().unwrap();
        let host = headers.get(HOST).unwrap();
        assert_eq!(*host, "custom-region".to_string());
    }

    #[test]
    fn test_path_style_url_ends_in_bucket() {
        // Test case without bucket in URL
        let region = "http://custom-region".parse().unwrap();
        let bucket = Bucket::new("foo", region, fake_credentials())
            .unwrap()
            .with_path_style();
        assert_eq!(bucket.url(), "http://custom-region/foo");

        // Test case with bucket in URL
        let region = "http://custom-region/foo".parse().unwrap();
        let bucket = Bucket::new("foo", region, fake_credentials())
            .unwrap()
            .with_path_style();
        assert_eq!(bucket.url(), "http://custom-region/foo");

        // Just to make sure...
        let region = "http://custom-region/foo".parse().unwrap();
        let bucket = Bucket::new("bar", region, fake_credentials())
            .unwrap()
            .with_path_style();
        assert_eq!(bucket.url(), "http://custom-region/foo/bar");
    }

    #[test]
    fn test_get_object_range_header() {
        let region = "http://custom-region".parse().unwrap();
        let bucket = Bucket::new("my-second-bucket", region, fake_credentials())
            .unwrap()
            .with_path_style();
        let path = "/my-second/path";

        let request = HyperRequest::new(
            &bucket,
            path,
            Command::GetObjectRange {
                start: 0,
                end: None,
            },
        )
        .unwrap();
        let headers = request.headers().unwrap();
        let range = headers.get(RANGE).unwrap();
        assert_eq!(range, "bytes=0-");

        let request = HyperRequest::new(
            &bucket,
            path,
            Command::GetObjectRange {
                start: 0,
                end: Some(1),
            },
        )
        .unwrap();
        let headers = request.headers().unwrap();
        let range = headers.get(RANGE).unwrap();
        assert_eq!(range, "bytes=0-1");
    }
}
