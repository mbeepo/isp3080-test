#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use nrf52833_hal::{
    gpio::{p0, p1, Level},
    pac::{CorePeripherals, Peripherals}, Delay,
};

#[entry]
fn main() -> ! {
    let core_peripherals = CorePeripherals::take().unwrap();
    let peripherals = Peripherals::take().unwrap();

    let p0 = p0::Parts::new(peripherals.P0);
    let p1 = p1::Parts::new(peripherals.P1);
    let mut delay = Delay::new(core_peripherals.SYST);

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
    
    let mut isp3080_nrf_pins = [
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

    loop {
        for pin in &mut isp3080_nrf_pins {
            pin.set_high().unwrap();
        }
        delay.delay_ms(1000);
        for pin in &mut isp3080_nrf_pins {
            pin.set_low().unwrap();
        }
        delay.delay_ms(1000);
    }
}