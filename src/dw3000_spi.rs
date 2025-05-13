use embassy_nrf::{gpio::Output, peripherals::SPI3, spim::{self, Spim}};
use embassy_time::Timer;
use embedded_hal_async::spi::{Operation, SpiDevice};
use embedded_hal::spi::SpiBus;

pub struct Dw3000Spi<'a> {
    inner: Spim<'a, SPI3>,
    cs_pin: Output<'a>,
}

impl<'a> Dw3000Spi<'a> {
    pub fn new(inner: Spim<'a, SPI3>, cs_pin: Output<'a>) -> Self {
        Self { inner, cs_pin }
    }
}

impl<'a> SpiDevice for Dw3000Spi<'a> {
    async fn transaction(&mut self, operations: &mut [embedded_hal::spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
        // Active low, high-to-low transition signals transaction start
        self.cs_pin.set_low();

        for op in operations {
            match op {
                Operation::DelayNs(nanos) => Timer::after_nanos(*nanos as u64).await,
                Operation::Read(words) => self.inner.read(words).await?,
                Operation::Write(words) => self.inner.write(words).await?,
                Operation::Transfer(read, write) => self.inner.transfer(read, write).await?,
                Operation::TransferInPlace(words) => self.inner.transfer_in_place(words).await?,
            }
        }

        self.inner.flush().unwrap();
        self.cs_pin.set_high();
        Ok(())
    }
}

impl<'a> embedded_hal::spi::ErrorType for Dw3000Spi<'a> {
    type Error = spim::Error;
}