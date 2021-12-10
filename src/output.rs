//! Single chain of 8-bit SIPO shift registers (e.g. 74HC595) for digital output

use core::cell::RefCell;

use embedded_hal::digital::v2::OutputPin;

use crate::{Error, Length};

////////////////////////////////////////////////////////////////////////////////

/// Trait to be implemented by chains that provide output pins.
pub trait SetOutput {
    /// Sets the output state for a pin.
    fn set_output(&mut self, pin: usize, state: bool) -> Result<(), Error>;

    /// Sets the output state for a pin without pin boundary checks.
    fn set_output_unchecked(&mut self, pin: usize, state: bool);
}

////////////////////////////////////////////////////////////////////////////////

/// Chain of SIPO shift registers.
pub struct Chain<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize> {
    /// Pin for the clock output signal.
    clock_pin: ClockPin,

    /// Pin for the latch output signal.
    latch_pin: LatchPin,

    /// Pin for the data output signal.
    data_pin: DataPin,

    /// Buffer storing the data to output.
    data_buffer: [u8; CHAIN_LENGTH],
}

impl<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize>
    Chain<ClockPin, LatchPin, DataPin, CHAIN_LENGTH>
where
    ClockPin: OutputPin,
    LatchPin: OutputPin,
    DataPin: OutputPin,
{
    /// Creates a new chain by consuming the pins.
    pub fn new(clock_pin: ClockPin, latch_pin: LatchPin, data_pin: DataPin) -> Self {
        Self {
            clock_pin,
            latch_pin,
            data_pin,
            data_buffer: [0; CHAIN_LENGTH],
        }
    }

    /// Frees the chain and returns the pins.
    pub fn free(self) -> (ClockPin, LatchPin, DataPin) {
        (self.clock_pin, self.latch_pin, self.data_pin)
    }

    /// Updates the chain by shifting the data from the buffer into the chips.
    pub fn update(&mut self) {
        self.latch_pin.set_low().ok();

        for data in self.data_buffer {
            for bit in 0..=7 {
                self.clock_pin.set_low().ok();

                let state = (data & (1 << (7 - bit))) != 0;

                if state {
                    self.data_pin.set_high().ok();
                } else {
                    self.data_pin.set_low().ok();
                }

                self.clock_pin.set_high().ok();
            }
        }

        self.latch_pin.set_high().ok();
    }
}

impl<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize> SetOutput
    for Chain<ClockPin, LatchPin, DataPin, CHAIN_LENGTH>
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
            self.data_buffer[index] |= 1 << bit;
        } else {
            self.data_buffer[index] &= !(1 << bit);
        }
    }
}

impl<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize> Length
    for Chain<ClockPin, LatchPin, DataPin, CHAIN_LENGTH>
{
    /// Returns the chain length.
    fn len(&self) -> usize {
        CHAIN_LENGTH
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Output pin of a chip in the chain.
pub struct Pin<'a, Chain> {
    /// Reference to the chain.
    chain: &'a RefCell<Chain>,

    /// Pin number of the output in the chain.
    pin: usize,
}

impl<'a, Chain> Pin<'a, Chain>
where
    Chain: SetOutput + Length,
{
    /// Creates a new output pin.
    pub fn new(chain: &'a RefCell<Chain>, pin: usize) -> Result<Self, Error> {
        if pin >= chain.borrow().len() * 8 {
            return Err(Error::PinOutOfRange);
        }

        Ok(Self { chain, pin })
    }
}

impl<'a, Chain> OutputPin for Pin<'a, Chain>
where
    Chain: SetOutput,
{
    type Error = Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.chain
            .borrow_mut()
            .set_output_unchecked(self.pin, false);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.chain.borrow_mut().set_output_unchecked(self.pin, true);
        Ok(())
    }
}
