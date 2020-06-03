# FastID-rs

[![Crates.io](https://img.shields.io/crates/v/fastid)](http://crates.io/crates/fastid)
[![Docs.rs](https://docs.rs/fastid/badge.svg)](https://docs.rs/fastid)
[![Crates.io](https://img.shields.io/crates/d/fastid)](http://crates.io/crates/fastid)
[![Crates.io](https://img.shields.io/crates/l/fastid)](https://github.com/JoyMoe/FastID-rs/blob/master/LICENSE)

Snowflake-like ID generating in Rust

## Usage

```rust
use fastid::FastIdWorker;

let mut worker = FastIdWorker::new(1);

let id = worker.next_id();

println!("{:#064b}", id);
println!("{}", id);
```

## License

The MIT License

More info see [LICENSE](LICENSE)
