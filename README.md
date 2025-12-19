# SNI breaker

We found that in a region that exists SNI block, some TLS packets are able to reach after thousands of trials. This program performs HTTPS requests concurrently until it succeeds.

## Build

Install the latest Rust toolchain, and build it by:
```shell
cargo build --release
```

## Example
Request `https://example.com` in 8 jobs until it succeeds.

```shell
sni_breaker -i -j 8 https://example.com
```

## Acknowledgement
* `reqwest`: An easy and powerful Rust HTTP Client
* `tokio`: A runtime for writing reliable asynchronous applications with Rust.
