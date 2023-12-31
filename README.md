# Pimoroni Unicorn

Rust port of the pimoroni unicorn devices.

For now, just the [galactic unicorn](https://shop.pimoroni.com/products/space-unicorns?variant=40842033561683) is WIP.

Each board will gain support for both the rp_hal and embassy crates. Examples can be found within each respective folder of how to make use of the library and the unicorn graphics library too.

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
