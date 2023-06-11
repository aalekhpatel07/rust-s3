# ZitaneLabs/rust-s3-async

![License: MIT](https://badgers.space/badge/license/MIT)

## Fork notice

This is a fork of [aalekhpatel07/rust-s3-async](https://github.com/aalekhpatel07/rust-s3-async), which is a fork of [durch/rust-s3](https://github.com/durch/rust-s3).

It's used in our internal and upcoming projects, and is not intended to be used by anyone else. It's not published on crates.io, and the API is not stable.

### Intro

Rust library for working with Amazon S3 or arbitrary S3 compatible APIs, fully compatible with `async/await`. Uses `tokio` under the hood.

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
