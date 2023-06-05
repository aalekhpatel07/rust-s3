[![](https://camo.githubusercontent.com/2fee3780a8605b6fc92a43dab8c7b759a274a6cf/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f72757374632d737461626c652d627269676874677265656e2e737667)](https://www.rust-lang.org/tools/install)
<!-- [![build](https://github.com/durch/rust-s3/workflows/build/badge.svg)](https://github.com/durch/rust-s3/actions)
[![](https://img.shields.io/crates/v/rust-s3.svg)](https://crates.io/crates/rust-s3)
![](https://img.shields.io/crates/d/rust-s3.svg) -->
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/durch/rust-s3/blob/master/LICENSE.md)
<!-- [![Join the chat at https://gitter.im/durch/rust-s3](https://badges.gitter.im/durch/rust-s3.svg)](https://gitter.im/durch/rust-s3?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge) -->


> **Note**: This repository is a fork of [rust-s3](https://github.com/durch/rust-s3) but with a lot of simplifications like only tokio support with async, no blocking implementations, and next to zero macro usage. The only reason this fork exists is because the macro expansions are fairly inaccurate and make development frustrating for me. Plus, its pretty common to use async code when consuming an S3 API, so I don't feel the need of carrying all the "mutual exclusive macro" dead weights around for every minor feature.
## rust-s3-async [[docs](https://docs.rs/rust-s3-async/latest/rust-s3-async)]

Rust library for working with Amazon S3 or arbitrary S3 compatible APIs, fully compatible with **async/await** and `futures ^0.3`.

### Intro

Modest interface towards Amazon S3, as well as S3 compatible object storage APIs such as Backblaze B2, Wasabi, Yandex, Minio or Google Cloud Storage.
Supports: `put`, `get`, `list`, `delete`, operations on `tags` and `location`, as well as `head`. 

Additionally, a dedicated `presign_get` `Bucket` method is available. This means you can upload to S3, and give the link to select people without having to worry about publicly accessible files on S3. This also means that you can give people 
a `PUT` presigned URL, meaning they can upload to a specific key in S3 for the duration of the presigned URL.

#### Quick Start

Read and run examples from the `examples` folder, make sure you have valid credentials for the variant you're running.

```bash
# tokio, default
cargo run --example tokio

# minio
# First, start Minio on port 9000.
AWS_ACCESS_KEY_ID="minioadmin" \
AWS_SECRET_ACCESS_KEY="minioadmin" \
    cargo run --example minio

# r2
cargo run --example r2

# google cloud
cargo run --example google-cloud
```

#### Features

There are a lot of various features that enable a wide variety of use cases, refer to `s3/Cargo.toml` for an exhaustive list. Below is a table of various useful features as well as a short description for each.

+ `default` - `tokio` runtime and a `native-tls` implementation
+ `fail-on-err` - `panic` on any error
+ `no-verify-ssl` - disable SSL verification for endpoints, useful for custom regions

#### Path or subdomain style URLs and headers

`Bucket` struct provides constructors for `path-style` paths, `subdomain` style is the default. `Bucket` exposes methods for configuring and accessing `path-style` configuration.

#### Buckets

|          |                                                                                         |
|----------|-----------------------------------------------------------------------------------------|
| `create` | [async](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.create)      |
| `delete` | [async](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.delete)      |
| `list`   | [async](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.list_buckets)|
| `exists` | [async](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.exists)|


#### Presign

|          |                                                                                                     |
|----------|-----------------------------------------------------------------------------------------------------|
| `POST`   | [presign_put](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.presign_post)      |
| `PUT`    | [presign_put](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.presign_put)       |
| `GET`    | [presign_get](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.presign_get)       |
| `DELETE` | [presign_delete](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.presign_delete) |

#### GET

There are a few different options for getting an object and methods are generic over `tokio::io::AsyncWriteExt`.

|                             |                                                                                                                 |
|-----------------------------|-----------------------------------------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [get_object](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.get_object)                     |
| `async/sync/async-blocking` | [get_object_stream](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.get_object_stream)       |
| `async/sync/async-blocking` | [get_object_to_writer](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.get_object_to_writer) |

#### PUT

Each `GET` method has a `PUT` companion, and `tokio` methods are generic over `tokio::io::AsyncReadExt`.

|                             |                                                                                                                                 |
|-----------------------------|---------------------------------------------------------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [put_object](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.put_object)                                     |
| `async/sync/async-blocking` | [put_object_with_content_type](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.put_object_with_content_type) |
| `async/sync/async-blocking` | [put_object_stream](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.put_object_stream)                       |

#### List

|                             |                                                                                 |
|-----------------------------|---------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [list](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.list) |

#### DELETE

|                             |                                                                                                   |
|-----------------------------|---------------------------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [delete_object](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.delete_object) |

#### Location

|                             |                                                                                         |
|-----------------------------|-----------------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [location](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.location) |

#### Tagging

|                             |                                                                                                             |
|-----------------------------|-------------------------------------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [put_object_tagging](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.put_object_tagging) |
| `async/sync/async-blocking` | [get_object_tagging](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.get_object_tagging) |

#### Head

|                             |                                                                                               |
|-----------------------------|-----------------------------------------------------------------------------------------------|
| `async/sync/async-blocking` | [head_object](https://docs.rs/rust-s3-async/latest/s3/bucket/struct.Bucket.html#method.head_object) |

### Usage (in `Cargo.toml`)

```toml
[dependencies]
rust-s3-async = "0.34.0"
```

