#![no_std]
#![no_main]

mod dw3000_spi;

use core::fmt::Display;

use log::info;
use panic_semihosting as _;

use dw3000_ng::{block, hl::{ConfigGPIOs, RxQuality}, time::Duration, DW3000};
use embassy_time::{Instant, Timer};
use dw3000_spi::Dw3000Spi;

use embassy_executor::{self, Spawner};
use embassy_nrf::{bind_interrupts, gpio::{self, Level, OutputDrive}, pac, peripherals::{self, USBD}, spim::{self, BitOrder, Frequency}, spis::MODE_0, usb::{self, vbus_detect::HardwareVbusDetect}};
use embassy_usb::{class::cdc_acm::{self, CdcAcmClass}, UsbDevice};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
});

const USB_VENDORID: u16 = 0xBABA;
const USB_PRODUCTID: u16 = 0xAAAA;
/// Speed of light in m/ns
const SPEED_OF_LIGHT: f32 = 0.299_792_458;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_nrf::init(Default::default());
    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}

    let driver = usb::Driver::new(peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    spawner.spawn(logger_task(driver)).unwrap();
    
    info!("[1/4] [USB] Running test\nIf you can read this, it works");
    info!("[1/4] [USB] Test Finished");

    // haha gpio block
    let mut gpio_pins = [
        gpio::Output::new(peripherals.P0_09, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_10, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_22, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_17, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_18, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_14, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_28, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_21, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_06, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_25, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_05, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_27, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_04, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_26, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_02, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_03, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_11, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_07, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P1_09, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_08, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P0_12, Level::Low, OutputDrive::Standard),
        gpio::Output::new(peripherals.P1_00, Level::Low, OutputDrive::Standard),
    ];

    info!("[2/4] [GPIO] Running test");
    info!("J5, J3, J6 HIGH, wait 1s, LOW, wait 1s");

    for pin in &mut gpio_pins {
        pin.set_high();
    }

    Timer::after_millis(1000).await;

    info!("[2/4] [GPIO] J5, J3, J6 LOW");
    for pin in &mut gpio_pins {
        pin.set_low();
    }

    Timer::after_millis(1000).await;

    info!("[2/4] [GPIO] Test finished");

    let mut spi_config = spim::Config::default();
    spi_config.frequency = Frequency::M32;
    spi_config.bit_order = BitOrder::MSB_FIRST;
    spi_config.mode = MODE_0;

    let spi = spim::Spim::new(peripherals.SPI3, Irqs, peripherals.P0_19, peripherals.P0_16, peripherals.P0_13, spi_config);
    let cs_pin = gpio::Output::new(peripherals.P0_20, Level::High, OutputDrive::Standard);
    let dw3000_spi = Dw3000Spi::new(spi, cs_pin);
    let dw3000 = DW3000::new(dw3000_spi);

    info!("[3/4] [DW3000 / GPIO] Running test");
    info!("J2 HIGH, wait 1s, LOW, wait 1s");

    let dw3000 = dw3000.init().await.unwrap();
    let mut dw3000 = dw3000.config(dw3000_ng::Config::default(), embassy_time::Delay).await.unwrap();
    
    dw3000.gpio_config_clocks().await.unwrap();
    dw3000.gpio_config(ConfigGPIOs {
        enabled: [1, 1, 1, 1, 1, 1, 1, 1, 1],
        mode: [0, 0, 0, 0, 0, 0, 0, 0, 0], // standard operating mode, only change when set by host
        gpio_dir: [0, 0, 0, 0, 0, 0, 0, 0, 0], // 0 for output, 1 for input
        output: [1, 1, 1, 1, 1, 1, 1, 1, 1],
    }).await.unwrap();

    Timer::after_millis(1000).await;

    info!("[3/4] [DW3000 / GPIO] J2 LOW");
    dw3000.gpio_config(ConfigGPIOs {
        enabled: [1, 1, 1, 1, 1, 1, 1, 1, 1],
        mode: [0, 0, 0, 0, 0, 0, 0, 0, 0], // standard operating mode, only change when set by host
        gpio_dir: [0, 0, 0, 0, 0, 0, 0, 0, 0], // 0 for output, 1 for input
        output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
    }).await.unwrap();

    Timer::after_millis(1000).await;
    info!("[3/4] [DW3000 / GPIO] Test finished");

    info!("[4/4] [DW3000 / RX/TX] Running test");
    info!("TX:RX 1:99");
    info!("This test will continue indefinitely");
    dw3000.set_full_cia_diagnostics(true).await.unwrap();

    loop {
        use dw3000_ng::{self, hl::{SendTime, ReceiveTime}, time::{self, Duration}, Config};
        let start_tx_time = Instant::now();
        let mut dw3000_sending = dw3000.send(&[], SendTime::Now, Config::default()).await.unwrap();
        // We can use DW3000's IRQ output to make this less power hungry (`block!` is a spinlock)
        let tx_time = block!(dw3000_sending.s_wait().await).unwrap();
        let tx_dur = start_tx_time.elapsed();

        dw3000 = dw3000_sending.finish_sending().await.unwrap();
        let dw_time = dw3000.sys_time().await.unwrap();
        let mut dw3000_receiving = dw3000.receive_delayed(
            ReceiveTime::Delayed(
                time::Instant::new(dw_time as u64).unwrap() + Duration::from_nanos(1000)
            ), Config::default()
        ).await.unwrap();
        let mut rx_data = [0u8; 64];

        let elapsed = Instant::now();
        let mut rx_time: Option<time::Instant> = None;
        loop {
            if elapsed.elapsed().as_micros() >= tx_dur.as_micros() * 99 {
                break;
            }

            if let Ok((_rx_len2, rx_time2, _rx_quality2)) = dw3000_receiving.r_wait_buf(&mut rx_data).await {
                rx_time = Some(rx_time2);

                info!("Data received: ");
                info!("{}", CharBufWriter::new(rx_data));

                break;
            }
        }
        
        if let Some(rx_time) = rx_time {
            const ADJUST_1: f32 = 67_108_864.0;
            const ADJUST_2: f32 = 1_000_000.0;
            let coe_ppm_raw: f32 = dw3000_receiving.ll().cia_diag_0().read().await.unwrap().coe_ppm().into();
            let coe_ppm = coe_ppm_raw / ADJUST_1 * ADJUST_2;
            
            // time in nanoseconds
            let prop_time = 0.5 * (as_nanos(&rx_time.duration_since(tx_time)) as f32 * (1.0 - coe_ppm));
            let distance = SPEED_OF_LIGHT * prop_time;
            info!("Distance: {distance}m");
        }
        dw3000 = dw3000_receiving.finish_receiving().await.unwrap();
    }
}

/// Formula from `dw3000_ng::time::Duration::from_nanos`
fn as_nanos(duration: &Duration) -> u64 {
    (duration.value() * 10000 - 5000) / 638976
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, usb::Driver<'static, USBD, HardwareVbusDetect>>) {
    device.run().await;
}

#[embassy_executor::task]
async fn logger_task(driver: usb::Driver<'static, USBD, HardwareVbusDetect>) {
   embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

struct CharBufWriter<const N: usize> {
    inner: [u8; N],
}

impl<const N: usize> CharBufWriter<N> {
    fn new(buf: [u8; N]) -> Self {
        Self { inner: buf }
    }
}

impl<const N: usize> Display for CharBufWriter<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for i in 0..N {
            if i == 0 {
                break;
            }
            write!(f, "{}", self.inner[i])?;
        }

        Ok(())
    }
}