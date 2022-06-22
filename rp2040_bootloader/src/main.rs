//! Bootloader jump to application
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

const FLASH_XIP_BASE: u32 = 0x1000_0000;
const FLASH_XIP_FIRMWARE_BASE: u32 = 0x8100;
const FLASH_XIP_APPLISTART_ADDR: u32 = FLASH_XIP_BASE + FLASH_XIP_FIRMWARE_BASE;

fn boot_from(address: u32) {
    static mut USER_RESET: Option<extern "C" fn()> = None;
    let scb = bsp::pac::SCB::ptr();

    unsafe {
        let sp = *(address as *const u32);
        let rv = *((address + 4) as *const u32);
        USER_RESET = Some(core::mem::transmute(rv));
        (*scb).vtor.write(address);
        cortex_m::register::msp::write(sp);
        (USER_RESET.unwrap())();
    }
}

#[entry]
fn main() -> ! {
    info!("Program starts");

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    for _x in 0..4 {
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(2000);
        info!("off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(2000);
    }

    boot_from(FLASH_XIP_APPLISTART_ADDR);
    loop {}
}

// End of file
