#![no_std]
#![no_main]

use dw3000_ng::DW3000;
use panic_semihosting as _;

use isp3080_test::dw3000_spi::Dw3000Spi;

use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiBus};
use nrf52833_hal::{
    gpio::{p0, p1, Level}, pac::{CorePeripherals, Peripherals}, spi::{self, Spi}, usbd::{UsbPeripheral, Usbd}, Clocks, Delay
};
use usb_device::{bus::UsbBusAllocator, device::{StringDescriptors, UsbDeviceBuilder, UsbVidPid}};
use usbd_serial::{embedded_io::Write, SerialPort};

const USB_VENDORID: u16 = 0xBABA;
const USB_PRODUCTID: u16 = 0xAAAA;

#[entry]
fn main() -> ! {
    // All this unwrapping is just because it's a test
    let core_peripherals = CorePeripherals::take().unwrap();
    let peripherals = Peripherals::take().unwrap();    let clocks = Clocks::new(peripherals.CLOCK);
    let clocks = clocks.enable_ext_hfosc();
    
    let p0 = p0::Parts::new(peripherals.P0);
    let p1 = p1::Parts::new(peripherals.P1);

    let p56 = p0.p0_09;
    let p57 = p0.p0_10;
    let p61 = p0.p0_22;
    let p65 = p0.p0_17;
    let p66 = p0.p0_18;
    let p67 = p0.p0_14;
    let p69 = p0.p0_28;
    let p73 = p0.p0_21;
    let p74 = p0.p0_06;
    let p75 = p0.p0_25;
    let p76 = p0.p0_05;
    let p77 = p0.p0_27;
    let p78 = p0.p0_04;
    let p79 = p0.p0_26;
    let p80 = p0.p0_02;
    let p82 = p0.p0_03;
    let p84 = p0.p0_11;
    let p86 = p0.p0_07;
    let p88 = p1.p1_09;
    let p90 = p0.p0_08;
    let p92 = p0.p0_12;
    let p94 = p1.p1_00;
    
    let mut gpio_pins = [
        p56.degrade(),
        p57.degrade(),
        p61.degrade(),
        p65.degrade(),
        p66.degrade(),
        p67.degrade(),
        p69.degrade(),
        p73.degrade(),
        p74.degrade(),
        p75.degrade(),
        p76.degrade(),
        p77.degrade(),
        p78.degrade(),
        p79.degrade(),
        p80.degrade(),
        p82.degrade(),
        p84.degrade(),
        p86.degrade(),
        p88.degrade(),
        p90.degrade(),
        p92.degrade(),
        p94.degrade(),
    ].map(|x| x.into_push_pull_output(Level::Low));
    

    // ISP3080 has DW3000 SPI pins connected to nRF in this configuration
    let sck = p0.p0_19.degrade().into_push_pull_output(Level::High);
    let mosi = p0.p0_16.degrade().into_push_pull_output(Level::Low);
    let miso = p0.p0_13.degrade().into_floating_input();
    let cs_pin = p0.p0_20.degrade().into_push_pull_output(Level::High);

    let spi_pins = spi::Pins {
        sck: Some(sck),
        mosi: Some(mosi),
        miso: Some(miso),
    };
    let mut spi = Spi::new(peripherals.SPI0, spi_pins, spi::Frequency::M8, embedded_hal::spi::MODE_0);
    let mut dw3000 = DW3000::new(Dw3000Spi { inner: spi, cs_pin, delay });

    let usb_bus = UsbBusAllocator::new(Usbd::new(UsbPeripheral::new(peripherals.USBD, &clocks)));
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(USB_VENDORID, USB_PRODUCTID))
        .strings(&[StringDescriptors::default()
            .manufacturer("Beepo")
            .product("Serial port")
            .serial_number("BABAWAWA")])
        .unwrap()
        .device_class(0xFF)
        .max_packet_size_0(8)
        .unwrap()
        .build();
    let mut serial = SerialPort::new(&usb_bus);
    let mut delay = Delay::new(core_peripherals.SYST);

    let mut test_step = 0;
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        match test_step {
            0 => {
                // hehe
                serial.write_all(b"[START] Test USB").unwrap();
                serial.write_all(b"[FINISH] Test USB").unwrap();
            }
            1 => {
                serial.write_all(b"[START] Test GPIO").unwrap();
                serial.write_all(b"J5,J3,J6 Header Groups HIGH").unwrap(); 
                for pin in &mut gpio_pins {
                    pin.set_high().unwrap();
                }

                // we might need to poll USB during this to avoid being declared dead but im not entirely sure
                delay.delay_ms(1000);

                for pin in &mut gpio_pins {
                    pin.set_low().unwrap();
                }
                serial.write_all(b"J5,J3,J6 Header Groups LOW").unwrap();
                serial.write_all(b"[FINISH] Test GPIO").unwrap();
            }
            2 => {
                serial.write_all(b"[START] Test GPIO").unwrap();
                serial.write_all(b"J2 Header Group HIGH").unwrap();

                spi.write(&[]).unwrap();

                serial.write_all(b"J2 Header Group LOW").unwrap();
                serial.write_all(b"[FINISH] Test GPIO").unwrap();
            }
            _ => {
                serial.write_all(b"-- All tests complete --").unwrap();
                loop {}
            }
        }

        test_step += 1;
    }
}