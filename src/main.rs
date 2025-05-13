#![no_std]
#![no_main]

mod dw3000_spi;

use panic_semihosting as _;

use dw3000_ng::{hl::ConfigGPIOs, DW3000};
use embassy_time::Timer;
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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_nrf::init(Default::default());
    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}

    let driver = usb::Driver::new(peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    let mut config = embassy_usb::Config::new(USB_VENDORID, USB_PRODUCTID);
    config.manufacturer = Some("beepo");
    config.product = Some("Serial port");
    config.serial_number = Some("BABAWAWA");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
    let state = STATE.init(cdc_acm::State::new());

    let mut serial = CdcAcmClass::new(&mut builder, state, 64);
    let usb = builder.build();
    spawner.spawn(usb_task(usb)).unwrap();
    
    serial.wait_connection().await;
    serial.write_packet(b"[1/4] [USB] Running test\nIf you can read this, it works").await.unwrap();
    serial.write_packet(b"[1/4] [USB] Test Finished").await.unwrap();

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

    serial.write_packet(b"[2/4] [GPIO] Running test\nJ5, J3, J6 HIGH, wait 1s, LOW, wait 1s").await.unwrap();

    for pin in &mut gpio_pins {
        pin.set_high();
    }

    Timer::after_millis(1000).await;

    serial.write_packet(b"[2/4] [GPIO] J5, J3, J6 LOW").await.unwrap();
    for pin in &mut gpio_pins {
        pin.set_low();
    }

    Timer::after_millis(1000).await;

    serial.write_packet(b"[2/4] [GPIO] Test finished").await.unwrap();

    let mut spi_config = spim::Config::default();
    spi_config.frequency = Frequency::M32;
    spi_config.bit_order = BitOrder::MSB_FIRST;
    spi_config.mode = MODE_0;

    let spi = spim::Spim::new(peripherals.SPI3, Irqs, peripherals.P0_19, peripherals.P0_16, peripherals.P0_13, spi_config);
    let cs_pin = gpio::Output::new(peripherals.P0_20, Level::High, OutputDrive::Standard);
    let dw3000_spi = Dw3000Spi::new(spi, cs_pin);
    let dw3000 = DW3000::new(dw3000_spi);

    serial.write_packet(b"[3/4] [DW3000 / GPIO] Running test\nJ2 HIGH, wait 1s, LOW, wait 1s").await.unwrap();

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

    serial.write_packet(b"[3/4] [DW3000 / GPIO] J2 LOW").await.unwrap();
    dw3000.gpio_config(ConfigGPIOs {
        enabled: [1, 1, 1, 1, 1, 1, 1, 1, 1],
        mode: [0, 0, 0, 0, 0, 0, 0, 0, 0], // standard operating mode, only change when set by host
        gpio_dir: [0, 0, 0, 0, 0, 0, 0, 0, 0], // 0 for output, 1 for input
        output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
    }).await.unwrap();

    Timer::after_millis(1000).await;
    serial.write_packet(b"[3/4] [DW3000 / GPIO] Test finished").await.unwrap();
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, usb::Driver<'static, USBD, HardwareVbusDetect>>) {
    device.run().await;
}