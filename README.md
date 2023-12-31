# Pimoroni Unicorn

Rust implementation of the pimoroni unicorn devices.

Each board will gain support for both the rp_hal and embassy crates.

## Current Features

### Galactic Unicorn

- [x] Display
- [x] Buttons
- [ ] Speaker
- [ ] Extensions

### Cosmic Unicorn

- [ ] Display
- [ ] Buttons
- [ ] Speaker
- [ ] Extensions

### Stellar Unicorn

- [ ] Display
- [ ] Buttons
- [ ] Speaker
- [ ] Extensions

## Unicorn Graphics

Holds a buffer of the led matrix 2d array used by the display. Benefits for using this library include:

- Hold multiple buffers of what can possibly be on the display
- Easily update the actual display buffer without loops
- Run comparisons against what is in the buffer, such as if it is colored or the same color as something else at a given pixel
- Support for the embedded graphics crate

## Examples

Examples can be found within each respective folder of how to make use of the library and the unicorn graphics library.

## Running examples

Install required dependencies.

```sh
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
cargo install probe-rs --features cli
```

> You may need Rust nightly if running the embassy examples. Run the above command again after running `rustup install nightly`.

Change directory to the sample's base directory.

For example:

`cd galactic-unicorn-rp/`

Run the example

For example:

`cargo run --release --example scrolling_text`

> Make sure your Pico is in boostel mode for this to succeed.
