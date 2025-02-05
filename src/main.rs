// traffic_light.rs

// stdと標準メイン関数を使用しない
#![no_std]
#![no_main]

// 宣言
// デバッグ出力
use defmt::*;
use defmt_rtt as _;
// panicハンドラ(panic発生時にデバッグヒントを出力)
use panic_probe as _;
// Raspberry Pi Picoのハードウェアアクセスライブラリ
use rp2040_hal as hal;

use hal::pac;
use hal::pio::PIOExt;

// 遅延関数
use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}};
// デジタル出力
// use embedded_hal::digital::OutputPin;

// ブートローダー宣言
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// 定数
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// メインの関数
#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    // ペリフェラル(マイコン内蔵機能)の取得
    let mut pac = pac::Peripherals::take().unwrap();
    //  ウォッチドッグの宣言
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    //  クロックの初期化
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

    //  タイマーの宣言
    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    //  SIO(Single-cycle I/O)の宣言
    let sio = hal::Sio::new(pac.SIO);

    //  GPIOの宣言
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // DHT11センサー

    let (dht_pio, dht_sm, ..) = pac.PIO0.split(&mut pac.RESETS);
    let mut dht = dht_pio::Dht11::new(dht_pio, dht_sm, pins.gpio16.into_function(), &clocks);
    let mut led_pin = pins.gpio25.into_push_pull_output();


    info!("DHT11 rp-pico");
    led_pin.set_high().unwrap();

    info!("waiting sensor");
    timer.delay_ms(2000);
    info!("done.");

    loop {
        match dht.read(&mut timer) {
            Ok(res) => {
                info!(
                    "tmp: {}, hum: {}",
                    res.temperature,
                    res.humidity
                );
            }
            Err(_) => {
                error!("DHT error: ");
            }
                
        }
        timer.delay_ms(2000);
        led_pin.toggle().unwrap();
    }
}
