#![no_std]
#![no_main]

mod fmt;

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{self, Level, Pull, Speed},
    spi::{Config, Spi},
};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_hal_bus::spi::RefCellDevice;
use epd_waveshare::{epd2in9::*, prelude::*};
use fmt::info;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mosi = p.PA7;
    let sck = p.PA5;
    let cs = p.PB0;
    let dc = p.PB1;
    let rst = p.PA0;
    let busy = p.PA1;

    let cs = gpio::Output::new(cs, Level::High, Speed::Medium);
    let dc = gpio::Output::new(dc, Level::Low, Speed::Medium);
    let rst = gpio::Output::new(rst, Level::High, Speed::Medium);
    let busy_in = gpio::Input::new(busy, Pull::None);

    let spi_bus = RefCell::new(Spi::new_txonly(
        p.SPI1,
        sck,
        mosi,
        p.DMA1_CH3,
        p.DMA1_CH2,
        Config::default(),
    ));

    let mut spi =
        RefCellDevice::new(&spi_bus, cs, embassy_time::Delay).expect("Spi dev creation failed");

    let mut delay = embassy_time::Delay;

    info!("Initializing epd");

    let mut epd =
        Epd2in9::new(&mut spi, busy_in, dc, rst, &mut delay, None).expect("EPD creation error");

    info!("Drawing");

    let mut mono_display = Display2in9::default();
    mono_display.set_rotation(DisplayRotation::Rotate90);

    let _ = Line::new(Point::new(0, 120), Point::new(0, 200))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(&mut mono_display);

    /*
    let mut chromatic_display = Display2in9::default();
    chromatic_display.set_rotation(DisplayRotation::Rotate90);

    let _ = Line::new(Point::new(15, 120), Point::new(15, 200))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(&mut chromatic_display);

    info!("Updating color frame");

    epd.update_color_frame(
        &mut spi,
        &mut delay,
        mono_display.buffer(),
        chromatic_display.buffer(),
    )
    .unwrap();
    */

    epd.display_frame(&mut spi, &mut delay).unwrap();

    info!("Pre sleep");

    epd.sleep(&mut spi, &mut delay).unwrap();

    info!("Post sleep");

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
