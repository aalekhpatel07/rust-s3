# ZitaneLabs/rust-s3-async

![License: MIT](https://badgers.space/badge/license/MIT)

## Fork notice

This is a fork of [aalekhpatel07/rust-s3-async](https://github.com/aalekhpatel07/rust-s3-async), which is a fork of [durch/rust-s3](https://github.com/durch/rust-s3).

It's used in our internal and upcoming projects, and is not intended to be used by anyone else. It's not published on crates.io, and the API is not stable.

## Introduction

Rust library for working with Amazon S3 or arbitrary S3 compatible APIs, fully compatible with `async/await`. Uses `tokio` under the hood.

## Quick Start

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

## Features

There are a lot of various features that enable a wide variety of use cases, refer to `s3/Cargo.toml` for an exhaustive list. 

+ `default` - `tokio` runtime and a `native-tls` implementation
+ `no-verify-ssl` - disable SSL verification for endpoints, useful for custom regions

### Path or subdomain style URLs and headers

`Bucket` struct provides constructors for `path-style` paths, `subdomain` style is the default. `Bucket` exposes methods for configuring and accessing `path-style` configuration.

## API

### Buckets

- `create`
- `delete`
- `list`
- `exists`
- `location`

### Objects

- `head_object`
- `get_object`
- `get_object_stream`
- `get_object_to_writer`
- `get_object_tagging`
- `put_object`
- `put_object_with_content_type`
- `put_object_stream`
- `put_object_tagging`
- `delete_object`
- `object_exists`

### Presign

- `presign_post`
- `presign_put`
- `presign_get`
- `presign_delete`
