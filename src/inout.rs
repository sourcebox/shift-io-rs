//! Dual chain of 8-bit PISO & SIPO shift registers (e.g. 74HC165/74HC595) for digital output

use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::{input::GetInput, output::SetOutput, Error, Length};

////////////////////////////////////////////////////////////////////////////////

/// Dual chain of SIPO/PISO shift registers.
pub struct DualChain<ClockPin, LatchPin, DataInPin, DataOutPin, const CHAIN_LENGTH: usize> {
    /// Pin for the clock output signal.
    clock_pin: ClockPin,

    /// Pin for the latch output signal.
    latch_pin: LatchPin,

    /// Pin for the data input signal.
    data_in_pin: DataInPin,

    /// Pin for the data output signal.
    data_out_pin: DataOutPin,

    /// Buffer storing the data read from pins.
    data_in_buffer: [u8; CHAIN_LENGTH],

    /// Buffer storing the data to output.
    data_out_buffer: [u8; CHAIN_LENGTH],
}

impl<ClockPin, LatchPin, DataInPin, DataOutPin, const CHAIN_LENGTH: usize>
    DualChain<ClockPin, LatchPin, DataInPin, DataOutPin, CHAIN_LENGTH>
where
    ClockPin: OutputPin,
    LatchPin: OutputPin,
    DataInPin: InputPin,
    DataOutPin: OutputPin,
{
    /// Creates a new chain by consuming the pins.
    pub fn new(
        clock_pin: ClockPin,
        latch_pin: LatchPin,
        data_in_pin: DataInPin,
        data_out_pin: DataOutPin,
    ) -> Self {
        Self {
            clock_pin,
            latch_pin,
            data_in_pin,
            data_out_pin,
            data_in_buffer: [0; CHAIN_LENGTH],
            data_out_buffer: [0; CHAIN_LENGTH],
        }
    }

    /// Frees the chain and returns the pins.
    pub fn free(self) -> (ClockPin, LatchPin, DataInPin, DataOutPin) {
        (
            self.clock_pin,
            self.latch_pin,
            self.data_in_pin,
            self.data_out_pin,
        )
    }

    /// Updates the chain inputs and outputs simultanously by shifting
    /// the data from and to the buffers.
    pub fn update(&mut self) {
        self.latch_pin.set_high().ok();

        for chain_index in 0..CHAIN_LENGTH {
            let mut in_value: u8 = 0;
            let out_value = self.data_out_buffer[chain_index];

            for bit in 0..=7 {
                self.clock_pin.set_low().ok();

                // Get input
                if self.data_in_pin.is_high().ok().unwrap() {
                    in_value |= 1 << (7 - bit);
                } else {
                    in_value &= !(1 << (7 - bit));
                }

                // Set output
                let out_state = (out_value & (1 << (7 - bit))) != 0;

                if out_state {
                    self.data_out_pin.set_high().ok();
                } else {
                    self.data_out_pin.set_low().ok();
                }

                self.clock_pin.set_high().ok();
            }

            self.data_in_buffer[chain_index] = in_value;
        }

        self.latch_pin.set_low().ok();

        // Additional latch cycle for output shift register to update
        // Otherwise, outputs would stay at previous states until next update() call
        self.latch_pin.set_high().ok();
        self.latch_pin.set_low().ok();
    }
}

impl<ClockPin, LatchPin, DataInPin, DataOutPin, const CHAIN_LENGTH: usize> GetInput
    for DualChain<ClockPin, LatchPin, DataInPin, DataOutPin, CHAIN_LENGTH>
{
    /// Returns the input state for a pin.
    ///
    /// The state is buffered and not read immediately because the bits
    /// have to be shifted in by calling `update()` first.
    fn get_input(&self, pin: usize) -> Result<bool, Error> {
        if pin >= CHAIN_LENGTH * 8 {
            return Err(Error::PinOutOfRange);
        }

        Ok(self.get_input_unchecked(pin))
    }

    /// Return the input state for a pin without pin boundary checks.
    ///
    /// The state is buffered and not read immediately because the bits
    /// have to be shifted in by calling `update()` first.
    fn get_input_unchecked(&self, pin: usize) -> bool {
        // Calculate index and bit position within buffer array
        let index = pin / 8;
        let bit = pin % 8;

        (self.data_in_buffer[index] & (1 << bit)) != 0
    }
}

impl<ClockPin, LatchPin, DataInPin, DataOutPin, const CHAIN_LENGTH: usize> SetOutput
    for DualChain<ClockPin, LatchPin, DataInPin, DataOutPin, CHAIN_LENGTH>
{
    /// Sets the output state for a pin.
    ///
    /// The output state is buffered and not set immediately because the bits
    /// have to be shifted out by calling `update()` first.
    fn set_output(&mut self, pin: usize, state: bool) -> Result<(), Error> {
        if pin >= CHAIN_LENGTH * 8 {
            return Err(Error::PinOutOfRange);
        }

        self.set_output_unchecked(pin, state);

        Ok(())
    }

    /// Sets the output state for a pin without pin boundary checks.
    ///
    /// The output state is buffered and not set immediately because the bits
    /// have to be shifted out by calling `update()` first.
    fn set_output_unchecked(&mut self, pin: usize, state: bool) {
        // Calculate index and bit position within buffer array
        let index = CHAIN_LENGTH - (pin / 8) - 1;
        let bit = pin % 8;

        if state {
            self.data_out_buffer[index] |= 1 << bit;
        } else {
            self.data_out_buffer[index] &= !(1 << bit);
        }
    }
}

impl<ClockPin, LatchPin, DataInPin, DataOutPin, const CHAIN_LENGTH: usize> Length
    for DualChain<ClockPin, LatchPin, DataInPin, DataOutPin, CHAIN_LENGTH>
{
    /// Returns the chain length.
    fn len(&self) -> usize {
        CHAIN_LENGTH
    }
}
