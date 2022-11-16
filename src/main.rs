#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use esp32c3_hal::{
    clock::ClockControl,
    gpio::Gpio9,
    gpio_types::{Event, Input, Pin, PullUp},
    interrupt,
    pac::{self, Peripherals},
    prelude::*,
    timer::TimerGroup,
    Rtc, IO, Delay
};
use esp_backtrace as _;

static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

#[riscv_rt::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    esp_println::println!("Hello World");

    // Set GPIO7 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio7.into_push_pull_output();
    led.set_high().unwrap();
    let mut delay = Delay::new(&clocks);

    let mut button = io.pins.gpio9.into_pull_up_input();
    button.listen(Event::FallingEdge); // raise interrupt on falling edge
    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));
    interrupt::enable(pac::Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    loop {
        // led.toggle().unwrap();
        led.set_high().unwrap();
        delay.delay_ms(100u32);
        led.set_low().unwrap();
        delay.delay_ms(1900u32);
        // if button.is_high().unwrap() {
        //     led.set_high().unwrap();
        //     delay.delay_ms(10u32);
        // } else {
        //     led.set_low().unwrap();
        //     delay.delay_ms(10u32);
        // }
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        esp_println::println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
