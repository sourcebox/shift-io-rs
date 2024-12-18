//! Single chain of 8-bit PISO shift registers (e.g. 74HC165) for digital input

use core::cell::RefCell;

use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

use crate::{Error, Length};

////////////////////////////////////////////////////////////////////////////////

/// Trait to be implemented by chains that provide input pins.
pub trait GetInput {
    /// Returns the input state for a pin.
    fn get_input(&self, pin: usize) -> Result<bool, Error>;

    /// Returns the input state for a pin without pin boundary checks.
    fn get_input_unchecked(&self, pin: usize) -> bool;
}

////////////////////////////////////////////////////////////////////////////////

/// Chain of PISO shift registers.
pub struct Chain<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize> {
    /// Pin for the clock output signal.
    clock_pin: ClockPin,

    /// Pin for the latch output signal.
    latch_pin: LatchPin,

    /// Pin for the data input signal.
    data_pin: DataPin,

    /// Buffer storing the data read from pins.
    data_buffer: [u8; CHAIN_LENGTH],
}

impl<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize>
    Chain<ClockPin, LatchPin, DataPin, CHAIN_LENGTH>
where
    ClockPin: OutputPin,
    LatchPin: OutputPin,
    DataPin: InputPin,
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

    /// Updates the chain by shifting the data from the chips into the buffer.
    pub fn update(&mut self) {
        self.latch_pin.set_high().ok();

        for data in self.data_buffer.iter_mut() {
            let mut value: u8 = 0;

            for bit in 0..=7 {
                self.clock_pin.set_low().ok();

                if self.data_pin.is_high().ok().unwrap() {
                    value |= 1 << (7 - bit);
                } else {
                    value &= !(1 << (7 - bit));
                }

                self.clock_pin.set_high().ok();
            }

            *data = value;
        }

        self.latch_pin.set_low().ok();
    }
}

impl<ClockPin, LatchPin, DataPin, const CHAIN_LENGTH: usize> GetInput
    for Chain<ClockPin, LatchPin, DataPin, CHAIN_LENGTH>
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

    /// Returns the input state for a pin without pin boundary checks.
    ///
    /// The state is buffered and not read immediately because the bits
    /// have to be shifted in by calling `update()` first.
    fn get_input_unchecked(&self, pin: usize) -> bool {
        // Calculate index and bit position within buffer array
        let index = pin / 8;
        let bit = pin % 8;

        (self.data_buffer[index] & (1 << bit)) != 0
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

/// Input pin of a chip in the chain.
pub struct Pin<'a, Chain> {
    /// Reference to the chain.
    chain: &'a RefCell<Chain>,

    /// Pin number of the input in the chain.
    pin: usize,
}

impl<'a, Chain> Pin<'a, Chain>
where
    Chain: GetInput + Length,
{
    /// Creates a new input pin.
    pub fn new(chain: &'a RefCell<Chain>, pin: usize) -> Result<Self, Error> {
        if pin >= chain.borrow().len() * 8 {
            return Err(Error::PinOutOfRange);
        }

        Ok(Self { chain, pin })
    }
}

impl<Chain> ErrorType for Pin<'_, Chain> {
    type Error = core::convert::Infallible;
}

impl<Chain> InputPin for Pin<'_, Chain>
where
    Chain: GetInput,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.chain.borrow().get_input_unchecked(self.pin))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.chain.borrow().get_input_unchecked(self.pin))
    }
}
