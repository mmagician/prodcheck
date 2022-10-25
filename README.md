<h1 align="center">Grand product check</h1>

<p align="center">
<a href="https://github.com/mmagician/prodcheck/actions?query=workflow%3ACI"><img src="https://github.com/mmagician/prodcheck/workflows/CI/badge.svg"></a>
    <a href="./LICENSE-APACHE"><img src="https://img.shields.io/badge/license-APACHE-blue.svg"></a>
   <a href="./LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
    <a href="https://deps.rs/repo/github/mmagician/prodcheck/"><img src="https://deps.rs/repo/github/mmagician/prodcheck/status.svg"></a>
</p>


`prodcheck` is a Rust library that implements the grand product check for multilinear polynomials. 


**WARNING**: This is an academic proof-of-concept prototype, and in particular has not received careful code review. This implementation is NOT ready for production use.

## Build guide

The library compiles on the `stable` toolchain of the Rust compiler. To install the latest version of Rust, first install `rustup` by following the instructions [here](https://rustup.rs/), or via your platform's package manager. Once `rustup` is installed, install the Rust toolchain by invoking:
```bash
rustup install stable
```

After that, use `cargo` (the standard Rust build tool) to build the library:
```bash
git clone https://github.com/mmagician/prodcheck.git
cd prodcheck
cargo build --release
```

This library comes with some unit and integration tests. Run these tests with:
```bash
cargo test
```

Lastly, this library is instrumented with profiling infrastructure that prints detailed traces of execution time. To enable this, compile with `cargo build --features print-trace`.

## License

This library is licensed under either of the following licenses, at your discretion.

* [Apache License Version 2.0](LICENSE-APACHE)
* [MIT License](LICENSE-MIT)

Unless you explicitly state otherwise, any contribution that you submit to this library shall be dual licensed as above (as defined in the Apache v2 License), without any additional terms or conditions.

## Reference Paper
[Quarks](https://eprint.iacr.org/2020/1275) <br/>
Srinath Setty and Jonathan Lee
