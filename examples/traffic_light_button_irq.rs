// traffic_light_button_irq.rs
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use hal::pac::interrupt;
use panic_probe as _;
use rp2040_hal as hal;

// bootloader code
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// グローバル変数の宣言
// LED
type GreenLedPin =
    hal::gpio::Pin<hal::gpio::bank0::Gpio22, hal::gpio::FunctionSioOutput, hal::gpio::PullDown>;
type RedLedPin =
    hal::gpio::Pin<hal::gpio::bank0::Gpio20, hal::gpio::FunctionSioOutput, hal::gpio::PullDown>;
type OrangeLedPin =
    hal::gpio::Pin<hal::gpio::bank0::Gpio21, hal::gpio::FunctionSioOutput, hal::gpio::PullDown>;
// ボタン
type ButtonPin =
    hal::gpio::Pin<hal::gpio::bank0::Gpio23, hal::gpio::FunctionSioInput, hal::gpio::PullUp>;
// タイマー
type DelayTimer = hal::Timer;
// ボタンとLEDのタプル
type LedAndButton = (GreenLedPin, RedLedPin, OrangeLedPin, ButtonPin, DelayTimer);
// グローバル変数の宣言(所有権の付与をする変数)
static GLOBAL_PINS: critical_section::Mutex<core::cell::RefCell<Option<LedAndButton>>> =
    critical_section::Mutex::new(core::cell::RefCell::new(None));

#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    let mut pac = hal::pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // LED:GPIO22(Green), GPIO21(orange), GPIO20(RED)
    let green_led = pins.gpio22.into_push_pull_output();
    let orange_led = pins.gpio21.into_push_pull_output();
    let mut red_led = pins.gpio20.into_push_pull_output();
    red_led.set_high().unwrap();

    // Button:GPIO23
    let button = pins.gpio23.into_pull_up_input();
    // 割り込みの有効化
    button.set_interrupt_enabled(hal::gpio::Interrupt::EdgeLow, true);

    // グローバル変数の宣言(所有権の付与をする変数)
    critical_section::with(|cs| {
        GLOBAL_PINS
            .borrow(cs)
            .replace(Some((green_led, red_led, orange_led, button, timer)));
    });

    // 割り込みの宣言
    // アンセーフブロックの宣言(割り込みの有効化に必要)
    unsafe {
        hal::pac::NVIC::unmask(hal::pac::Interrupt::IO_IRQ_BANK0);
    }

    // info!("red");
    loop {
        // 割り込みの待機(Wait for Interrupt)
        cortex_m::asm::wfi();
    }
}

// 割り込みの宣言
#[hal::pac::interrupt]
fn IO_IRQ_BANK0() {
    static mut LED_AND_BUTTON: Option<LedAndButton> = None;

    // 変数が初期化状態の場合
    if LED_AND_BUTTON.is_none() {
        // 変数の操作(所有権の付与をする変数)
        critical_section::with(|cs| {
            *LED_AND_BUTTON = GLOBAL_PINS.borrow(cs).take();
        });
    }

    // 変数が初期化されている場合
    if let Some(gpios) = LED_AND_BUTTON {
        // タプルの分解
        let (green_led, red_led, orange_led, button, timer) = gpios;
        // ボタンが押された場合
        if button.interrupt_status(hal::gpio::Interrupt::EdgeLow) {
            info!("Button pressed");

            red_led.set_low().unwrap();

            info!("green");
            green_led.set_high().unwrap();
            timer.delay_ms(2000);
            green_led.set_low().unwrap();

            info!("orange");
            for _ in 1..4 {
                orange_led.set_high().unwrap();
                timer.delay_ms(500);
                orange_led.set_low().unwrap();
                timer.delay_ms(500);
            }
            orange_led.set_low().unwrap();

            info!("red");
            red_led.set_high().unwrap();
            button.clear_interrupt(hal::gpio::Interrupt::EdgeLow);
        }
    }
}
