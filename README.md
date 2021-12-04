# shift-io

This Rust crate implements a digital i/o shift register driver that supports:

- A single chain of parallel in serial out shift registers (PISO) of type 74HC165 or alike.
- A single chain of serial in parallel out shift registers (SIPO) of type 74HC595 or alike.
- A dual chain of the above using common clock and latch signals.

**WARNING:** This crate is WIP and may not work as expected! Examples are also not tested any will possibly not even compile.

## Usage Examples

### Single chain of output shift registers

```rust
// Number of chips in the chain
const CHAIN_LENGTH: usize = 8;

// Defining a own type for the chain makes it easier to pass it around.
type OutputChain =
    shift_io::output::Chain<PA0<Output<PushPull>>, PA1<Output<PushPull>>, 
    PA2<Output<PushPull>>, CHAIN_LENGTH>;

// Initialize pins, code may vary depending on the HAL used
let clock_pin = gpioa
    .pa0
    .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
let latch_pin = gpioa
    .pa1
    .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
let data_out_pin = gpioa
    .pa2

// Create a new chain
let output_chain: OutputChain =
    shift_io::output::Chain::new(clock_pin, latch_pin, data_out_pin);

// Put chain into a RefCell to allow several mutable borrows
let output_chain_refcell = RefCell::new(output_chain);

// Make some pin objects. These implement the OutputPin trait and can
// be passed to anything that accepts this trait.
// The pin argument must be in range 0 to CHAIN_LENGTH-1.
// Numbering starts from output Q0 of the chip that is first in the chain
let output_pin1 = shift_io::output::Pin::new(&output_chain_refcell, 0);
let output_pin5 = shift_io::output::Pin::new(&output_chain_refcell, 5);

// Set the output state for the pins.
// The states are not immediately updated but written into a buffer
output_pin1.set_high().ok();
output_pin5.set_low().ok();

// Shift out the states from the buffer.
output_chain_refcell.borrow_mut().update();
```

## License

Published under the MIT license.

Author: Oliver Rockstedt <info@sourcebox.de>