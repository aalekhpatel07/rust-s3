#[cfg(feature = "blocking")]
use block_on_proc::block_on;
#[cfg(feature = "tags")]
use minidom::Element;
use std::collections::HashMap;
use std::time::Duration;

use crate::bucket_ops::{BucketConfiguration, CreateBucketResponse};
use crate::command::{Command, Multipart};
use crate::creds::Credentials;
use crate::region::Region;
use crate::request::ResponseData;

#[cfg(any(feature = "with-tokio", feature = "with-async-std"))]
use crate::request::ResponseDataStream;

use std::str::FromStr;
use std::sync::{Arc, RwLock};


#[cfg(test)]
mod test {

    use crate::creds::Credentials;
    use crate::region::Region;
    use crate::serde_types::CorsConfiguration;
    use crate::serde_types::CorsRule;
    use crate::Bucket;
    use crate::BucketConfiguration;
    use crate::Tag;
    use http::header::HeaderName;
    use http::HeaderMap;
    use std::env;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn test_aws_credentials() -> Credentials {
        Credentials::new(
            Some(&env::var("EU_AWS_ACCESS_KEY_ID").unwrap()),
            Some(&env::var("EU_AWS_SECRET_ACCESS_KEY").unwrap()),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn test_gc_credentials() -> Credentials {
        Credentials::new(
            Some(&env::var("GC_ACCESS_KEY_ID").unwrap()),
            Some(&env::var("GC_SECRET_ACCESS_KEY").unwrap()),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn test_wasabi_credentials() -> Credentials {
        Credentials::new(
            Some(&env::var("WASABI_ACCESS_KEY_ID").unwrap()),
            Some(&env::var("WASABI_SECRET_ACCESS_KEY").unwrap()),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn test_minio_credentials() -> Credentials {
        Credentials::new(Some("test"), Some("test1234"), None, None, None).unwrap()
    }

    fn test_digital_ocean_credentials() -> Credentials {
        Credentials::new(
            Some(&env::var("DIGITAL_OCEAN_ACCESS_KEY_ID").unwrap()),
            Some(&env::var("DIGITAL_OCEAN_SECRET_ACCESS_KEY").unwrap()),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn test_r2_credentials() -> Credentials {
        Credentials::new(
            Some(&env::var("R2_ACCESS_KEY_ID").unwrap()),
            Some(&env::var("R2_SECRET_ACCESS_KEY").unwrap()),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn test_aws_bucket() -> Bucket {
        Bucket::new(
            "rust-s3-test",
            "eu-central-1".parse().unwrap(),
            test_aws_credentials(),
        )
        .unwrap()
    }

    fn test_wasabi_bucket() -> Bucket {
        Bucket::new(
            "rust-s3",
            "wa-eu-central-1".parse().unwrap(),
            test_wasabi_credentials(),
        )
        .unwrap()
    }

    fn test_gc_bucket() -> Bucket {
        let mut bucket = Bucket::new(
            "rust-s3",
            Region::Custom {
                region: "us-east1".to_owned(),
                endpoint: "https://storage.googleapis.com".to_owned(),
            },
            test_gc_credentials(),
        )
        .unwrap();
        bucket.set_listobjects_v1();
        bucket
    }

    fn test_minio_bucket() -> Bucket {
        Bucket::new(
            "rust-s3",
            Region::Custom {
                region: "eu-central-1".to_owned(),
                endpoint: "http://localhost:9000".to_owned(),
            },
            test_minio_credentials(),
        )
        .unwrap()
        .with_path_style()
    }

    fn test_digital_ocean_bucket() -> Bucket {
        Bucket::new("rust-s3", Region::DoFra1, test_digital_ocean_credentials()).unwrap()
    }

    fn test_r2_bucket() -> Bucket {
        Bucket::new(
            "rust-s3",
            Region::R2 {
                account_id: "f048f3132be36fa1aaa8611992002b3f".to_string(),
            },
            test_r2_credentials(),
        )
        .unwrap()
    }

    fn object(size: u32) -> Vec<u8> {
        (0..size).map(|_| 33).collect()
    }

    #[maybe_async::maybe_async]
    async fn put_head_get_delete_object(bucket: Bucket, head: bool) {
        let s3_path = "/+test.file";
        let test: Vec<u8> = object(3072);

        let response_data = bucket.put_object(s3_path, &test).await.unwrap();
        assert_eq!(response_data.status_code(), 200);
        let response_data = bucket.get_object(s3_path).await.unwrap();
        assert_eq!(response_data.status_code(), 200);
        assert_eq!(test, response_data.as_slice());

        let response_data = bucket
            .get_object_range(s3_path, 100, Some(1000))
            .await
            .unwrap();
        assert_eq!(response_data.status_code(), 206);
        assert_eq!(test[100..1001].to_vec(), response_data.as_slice());
        if head {
            let (head_object_result, code) = bucket.head_object(s3_path).await.unwrap();
            assert_eq!(code, 200);
            assert_eq!(
                head_object_result.content_type.unwrap(),
                "application/octet-stream".to_owned()
            );
        }

        // println!("{:?}", head_object_result);
        let response_data = bucket.delete_object(s3_path).await.unwrap();
        assert_eq!(response_data.status_code(), 204);
    }

    #[ignore]
    #[cfg(feature = "tags")]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn test_tagging_aws() {
        let bucket = test_aws_bucket();
        let _target_tags = vec![
            Tag {
                key: "Tag1".to_string(),
                value: "Value1".to_string(),
            },
            Tag {
                key: "Tag2".to_string(),
                value: "Value2".to_string(),
            },
        ];
        let empty_tags: Vec<Tag> = Vec::new();
        let response_data = bucket
            .put_object("tagging_test", b"Gimme tags")
            .await
            .unwrap();
        assert_eq!(response_data.status_code(), 200);
        let (tags, _code) = bucket.get_object_tagging("tagging_test").await.unwrap();
        assert_eq!(tags, empty_tags);
        let response_data = bucket
            .put_object_tagging("tagging_test", &[("Tag1", "Value1"), ("Tag2", "Value2")])
            .await
            .unwrap();
        assert_eq!(response_data.status_code(), 200);
        // This could be eventually consistent now
        let (_tags, _code) = bucket.get_object_tagging("tagging_test").await.unwrap();
        // assert_eq!(tags, target_tags)
        let _response_data = bucket.delete_object("tagging_test").await.unwrap();
    }

    #[ignore]
    #[cfg(feature = "tags")]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn test_tagging_minio() {
        let bucket = test_minio_bucket();
        let _target_tags = vec![
            Tag {
                key: "Tag1".to_string(),
                value: "Value1".to_string(),
            },
            Tag {
                key: "Tag2".to_string(),
                value: "Value2".to_string(),
            },
        ];
        let empty_tags: Vec<Tag> = Vec::new();
        let response_data = bucket
            .put_object("tagging_test", b"Gimme tags")
            .await
            .unwrap();
        assert_eq!(response_data.status_code(), 200);
        let (tags, _code) = bucket.get_object_tagging("tagging_test").await.unwrap();
        assert_eq!(tags, empty_tags);
        let response_data = bucket
            .put_object_tagging("tagging_test", &[("Tag1", "Value1"), ("Tag2", "Value2")])
            .await
            .unwrap();
        assert_eq!(response_data.status_code(), 200);
        // This could be eventually consistent now
        let (_tags, _code) = bucket.get_object_tagging("tagging_test").await.unwrap();
        // assert_eq!(tags, target_tags)
        let _response_data = bucket.delete_object("tagging_test").await.unwrap();
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_big_aws_put_head_get_delete_object() {
        streaming_test_put_get_delete_big_object(test_aws_bucket()).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(
            all(
                not(feature = "sync"),
                not(feature = "tokio-rustls-tls"),
                feature = "with-tokio"
            ),
            tokio::test
        ),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_big_gc_put_head_get_delete_object() {
        streaming_test_put_get_delete_big_object(test_gc_bucket()).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_big_minio_put_head_get_delete_object() {
        streaming_test_put_get_delete_big_object(test_minio_bucket()).await;
    }

    // Test multi-part upload
    #[maybe_async::maybe_async]
    async fn streaming_test_put_get_delete_big_object(bucket: Bucket) {
        #[cfg(feature = "with-async-std")]
        use async_std::fs::File;
        #[cfg(feature = "with-async-std")]
        use async_std::io::WriteExt;
        #[cfg(feature = "with-async-std")]
        use async_std::stream::StreamExt;
        #[cfg(feature = "with-tokio")]
        use futures::StreamExt;
        #[cfg(not(any(feature = "with-tokio", feature = "with-async-std")))]
        use std::fs::File;
        #[cfg(not(any(feature = "with-tokio", feature = "with-async-std")))]
        use std::io::Write;
        #[cfg(feature = "with-tokio")]
        use tokio::fs::File;
        #[cfg(feature = "with-tokio")]
        use tokio::io::AsyncWriteExt;

        init();
        let remote_path = "+stream_test_big";
        let local_path = "+stream_test_big";
        std::fs::remove_file(remote_path).unwrap_or(());
        let content: Vec<u8> = object(20_000_000);

        let mut file = File::create(local_path).await.unwrap();
        file.write_all(&content).await.unwrap();
        let mut reader = File::open(local_path).await.unwrap();

        let response = bucket
            .put_object_stream(&mut reader, remote_path)
            .await
            .unwrap();
        assert_eq!(response.status_code(), 200);
        let mut writer = Vec::new();
        let code = bucket
            .get_object_to_writer(remote_path, &mut writer)
            .await
            .unwrap();
        assert_eq!(code, 200);
        // assert_eq!(content, writer);
        assert_eq!(content.len(), writer.len());
        assert_eq!(content.len(), 20_000_000);

        #[cfg(any(feature = "with-tokio", feature = "with-async-std"))]
        {
            let mut response_data_stream = bucket.get_object_stream(remote_path).await.unwrap();

            let mut bytes = vec![];

            while let Some(chunk) = response_data_stream.bytes().next().await {
                bytes.push(chunk)
            }
            assert_ne!(bytes.len(), 0);
        }

        let response_data = bucket.delete_object(remote_path).await.unwrap();
        assert_eq!(response_data.status_code(), 204);
        std::fs::remove_file(local_path).unwrap_or(());
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_aws_put_head_get_delete_object() {
        streaming_test_put_get_delete_small_object(test_aws_bucket()).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(
            all(
                not(feature = "sync"),
                not(feature = "tokio-rustls-tls"),
                feature = "with-tokio"
            ),
            tokio::test
        ),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_gc_put_head_get_delete_object() {
        streaming_test_put_get_delete_small_object(test_gc_bucket()).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_r2_put_head_get_delete_object() {
        streaming_test_put_get_delete_small_object(test_r2_bucket()).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn streaming_minio_put_head_get_delete_object() {
        streaming_test_put_get_delete_small_object(test_minio_bucket()).await;
    }

    #[maybe_async::maybe_async]
    async fn streaming_test_put_get_delete_small_object(bucket: Bucket) {
        init();
        let remote_path = "+stream_test_small";
        let content: Vec<u8> = object(1000);
        #[cfg(feature = "with-tokio")]
        let mut reader = std::io::Cursor::new(&content);
        #[cfg(feature = "with-async-std")]
        let mut reader = async_std::io::Cursor::new(&content);

        let response = bucket
            .put_object_stream(&mut reader, remote_path)
            .await
            .unwrap();
        assert_eq!(response.status_code(), 200);
        let mut writer = Vec::new();
        let code = bucket
            .get_object_to_writer(remote_path, &mut writer)
            .await
            .unwrap();
        assert_eq!(code, 200);
        assert_eq!(content, writer);

        let response_data = bucket.delete_object(remote_path).await.unwrap();
        assert_eq!(response_data.status_code(), 204);
    }

    #[cfg(feature = "blocking")]
    fn put_head_get_list_delete_object_blocking(bucket: Bucket) {
        let s3_path = "/test_blocking.file";
        let s3_path_2 = "/test_blocking.file2";
        let s3_path_3 = "/test_blocking.file3";
        let test: Vec<u8> = object(3072);

        // Test PutObject
        let response_data = bucket.put_object_blocking(s3_path, &test).unwrap();
        assert_eq!(response_data.status_code(), 200);

        // Test GetObject
        let response_data = bucket.get_object_blocking(s3_path).unwrap();
        assert_eq!(response_data.status_code(), 200);
        assert_eq!(test, response_data.as_slice());

        // Test GetObject with a range
        let response_data = bucket
            .get_object_range_blocking(s3_path, 100, Some(1000))
            .unwrap();
        assert_eq!(response_data.status_code(), 206);
        assert_eq!(test[100..1001].to_vec(), response_data.as_slice());

        // Test HeadObject
        let (head_object_result, code) = bucket.head_object_blocking(s3_path).unwrap();
        assert_eq!(code, 200);
        assert_eq!(
            head_object_result.content_type.unwrap(),
            "application/octet-stream".to_owned()
        );
        // println!("{:?}", head_object_result);

        // Put some additional objects, so that we can test ListObjects
        let response_data = bucket.put_object_blocking(s3_path_2, &test).unwrap();
        assert_eq!(response_data.status_code(), 200);
        let response_data = bucket.put_object_blocking(s3_path_3, &test).unwrap();
        assert_eq!(response_data.status_code(), 200);

        // Test ListObjects, with continuation
        let (result, code) = bucket
            .list_page_blocking(
                "test_blocking.".to_string(),
                Some("/".to_string()),
                None,
                None,
                Some(2),
            )
            .unwrap();
        assert_eq!(code, 200);
        assert_eq!(result.contents.len(), 2);
        assert_eq!(result.contents[0].key, s3_path[1..]);
        assert_eq!(result.contents[1].key, s3_path_2[1..]);

        let cont_token = result.next_continuation_token.unwrap();

        let (result, code) = bucket
            .list_page_blocking(
                "test_blocking.".to_string(),
                Some("/".to_string()),
                Some(cont_token),
                None,
                Some(2),
            )
            .unwrap();
        assert_eq!(code, 200);
        assert_eq!(result.contents.len(), 1);
        assert_eq!(result.contents[0].key, s3_path_3[1..]);
        assert!(result.next_continuation_token.is_none());

        // cleanup (and test Delete)
        let response_data = bucket.delete_object_blocking(s3_path).unwrap();
        assert_eq!(code, 200);
        let response_data = bucket.delete_object_blocking(s3_path_2).unwrap();
        assert_eq!(code, 200);
        let response_data = bucket.delete_object_blocking(s3_path_3).unwrap();
        assert_eq!(code, 200);
    }

    #[ignore]
    #[cfg(all(
        any(feature = "with-tokio", feature = "with-async-std"),
        feature = "blocking"
    ))]
    #[test]
    fn aws_put_head_get_delete_object_blocking() {
        put_head_get_list_delete_object_blocking(test_aws_bucket())
    }

    #[ignore]
    #[cfg(all(
        any(feature = "with-tokio", feature = "with-async-std"),
        feature = "blocking"
    ))]
    #[test]
    fn gc_put_head_get_delete_object_blocking() {
        put_head_get_list_delete_object_blocking(test_gc_bucket())
    }

    #[ignore]
    #[cfg(all(
        any(feature = "with-tokio", feature = "with-async-std"),
        feature = "blocking"
    ))]
    #[test]
    fn wasabi_put_head_get_delete_object_blocking() {
        put_head_get_list_delete_object_blocking(test_wasabi_bucket())
    }

    #[ignore]
    #[cfg(all(
        any(feature = "with-tokio", feature = "with-async-std"),
        feature = "blocking"
    ))]
    #[test]
    fn minio_put_head_get_delete_object_blocking() {
        put_head_get_list_delete_object_blocking(test_minio_bucket())
    }

    #[ignore]
    #[cfg(all(
        any(feature = "with-tokio", feature = "with-async-std"),
        feature = "blocking"
    ))]
    #[test]
    fn digital_ocean_put_head_get_delete_object_blocking() {
        put_head_get_list_delete_object_blocking(test_digital_ocean_bucket())
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn aws_put_head_get_delete_object() {
        put_head_get_delete_object(test_aws_bucket(), true).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(
            all(
                not(any(feature = "sync", feature = "tokio-rustls-tls")),
                feature = "with-tokio"
            ),
            tokio::test
        ),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn gc_test_put_head_get_delete_object() {
        put_head_get_delete_object(test_gc_bucket(), true).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn wasabi_test_put_head_get_delete_object() {
        put_head_get_delete_object(test_wasabi_bucket(), true).await;
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn minio_test_put_head_get_delete_object() {
        put_head_get_delete_object(test_minio_bucket(), true).await;
    }

    // Keeps failing on tokio-rustls-tls
    // #[ignore]
    // #[maybe_async::test(
    //     feature = "sync",
    //     async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
    //     async(
    //         all(not(feature = "sync"), feature = "with-async-std"),
    //         async_std::test
    //     )
    // )]
    // async fn digital_ocean_test_put_head_get_delete_object() {
    //     put_head_get_delete_object(test_digital_ocean_bucket(), true).await;
    // }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn r2_test_put_head_get_delete_object() {
        put_head_get_delete_object(test_r2_bucket(), false).await;
    }

    #[test]
    #[ignore]
    fn test_presign_put() {
        let s3_path = "/test/test.file";
        let bucket = test_aws_bucket();

        let mut custom_headers = HeaderMap::new();
        custom_headers.insert(
            HeaderName::from_static("custom_header"),
            "custom_value".parse().unwrap(),
        );

        let url = bucket
            .presign_put(s3_path, 86400, Some(custom_headers))
            .unwrap();

        assert!(url.contains("custom_header%3Bhost"));
        assert!(url.contains("/test/test.file"))
    }

    #[test]
    #[ignore]
    fn test_presign_get() {
        let s3_path = "/test/test.file";
        let bucket = test_aws_bucket();

        let url = bucket.presign_get(s3_path, 86400, None).unwrap();
        assert!(url.contains("/test/test.file?"))
    }

    #[test]
    #[ignore]
    fn test_presign_delete() {
        let s3_path = "/test/test.file";
        let bucket = test_aws_bucket();

        let url = bucket.presign_delete(s3_path, 86400).unwrap();
        assert!(url.contains("/test/test.file?"))
    }

    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    #[ignore]
    async fn test_bucket_create_delete_default_region() {
        let config = BucketConfiguration::default();
        let response = Bucket::create(
            &uuid::Uuid::new_v4().to_string(),
            "us-east-1".parse().unwrap(),
            test_aws_credentials(),
            config,
        )
        .await
        .unwrap();

        assert_eq!(&response.response_text, "");

        assert_eq!(response.response_code, 200);

        let response_code = response.bucket.delete().await.unwrap();
        assert!(response_code < 300);
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn test_bucket_create_delete_non_default_region() {
        let config = BucketConfiguration::default();
        let response = Bucket::create(
            &uuid::Uuid::new_v4().to_string(),
            "eu-central-1".parse().unwrap(),
            test_aws_credentials(),
            config,
        )
        .await
        .unwrap();

        assert_eq!(&response.response_text, "");

        assert_eq!(response.response_code, 200);

        let response_code = response.bucket.delete().await.unwrap();
        assert!(response_code < 300);
    }

    #[ignore]
    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    async fn test_bucket_create_delete_non_default_region_public() {
        let config = BucketConfiguration::public();
        let response = Bucket::create(
            &uuid::Uuid::new_v4().to_string(),
            "eu-central-1".parse().unwrap(),
            test_aws_credentials(),
            config,
        )
        .await
        .unwrap();

        assert_eq!(&response.response_text, "");

        assert_eq!(response.response_code, 200);

        let response_code = response.bucket.delete().await.unwrap();
        assert!(response_code < 300);
    }

    #[test]
    fn test_tag_has_key_and_value_functions() {
        let key = "key".to_owned();
        let value = "value".to_owned();
        let tag = Tag { key, value };
        assert_eq!["key", tag.key()];
        assert_eq!["value", tag.value()];
    }

    #[test]
    #[ignore]
    fn test_builder_composition() {
        use std::time::Duration;

        let bucket = Bucket::new(
            "test-bucket",
            "eu-central-1".parse().unwrap(),
            test_aws_credentials(),
        )
        .unwrap()
        .with_request_timeout(Duration::from_secs(10));

        assert_eq!(bucket.request_timeout(), Some(Duration::from_secs(10)));
    }

    #[maybe_async::test(
        feature = "sync",
        async(all(not(feature = "sync"), feature = "with-tokio"), tokio::test),
        async(
            all(not(feature = "sync"), feature = "with-async-std"),
            async_std::test
        )
    )]
    #[ignore]
    async fn test_put_bucket_cors() {
        let bucket = test_aws_bucket();
        let rule = CorsRule::new(
            None,
            vec!["GET".to_string()],
            vec!["*".to_string()],
            None,
            None,
            None,
        );
        let cors_config = CorsConfiguration::new(vec![rule]);
        let response = bucket.put_bucket_cors(cors_config).await.unwrap();
        assert_eq!(response.status_code(), 200)
    }
}
