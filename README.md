# Pimoroni Unicorn

Rust implementation of the pimoroni unicorn devices.

Current support is focused on the galactic unicorn with embassy.

There is a basic working version of the galactic unicorn with the rp_hal crate.

## Current Features

### Galactic Unicorn

- [x] Display
- [x] Buttons
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
