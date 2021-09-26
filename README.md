# webusb

Implementation of the [WebUSB specification](https://wicg.github.io/webusb/) in
Rust.

[![Documentation](https://docs.rs/webusb/badge.svg)](https://docs.rs/webusb)
[![Package](https://img.shields.io/crates/v/webusb.svg)](https://crates.io/crates/webusb)
[![Coverage Status](https://coveralls.io/repos/github/littledivy/webusb/badge.svg)](https://coveralls.io/github/littledivy/webusb)

```toml
[dependencies]
webusb = "0.3.0"
```

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/X8X4Y6IZ)

### Async

The methods are marked as `async` for compability between the native and wasm target.

### Testing

Hardware tests are run before merging a PR and then on `main`. The test runner is a self-hosted Linux x86_64 machine, it is connected to an Arduino Leonardo (ATmega32u4) via micro USB connection.

Tests are reviewed and triggered by maintainers on PRs to prevent malicious execution. Load [this sketch](https://github.com/webusb/arduino/blob/gh-pages/demos/console/sketch/sketch.ino) into yours to run the tests locally.

When writing tests you might encounter frequent Io / NoDevice errors, this can be due to loose wired connection. Mark these tests as `#[flaky_test::flaky_test]`.

### License

MIT License
