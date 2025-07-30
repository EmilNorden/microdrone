use embassy_time::{Duration, Timer};
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb565,
    mono_font::MonoTextStyle,
    mono_font::ascii::FONT_6X10,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text
};
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::{
    delay::Delay,
    Async,
    gpio::Output,
    spi::master::Spi
};
use ssd1351::{
    mode::GraphicsMode,
    prelude::SPIInterface,
    properties::DisplayRotation,
    properties::DisplaySize::Display128x128
};
use crate::input::get_controller_state;

#[embassy_executor::task]
pub async fn run(spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>, mut rst: Output<'static>, dc: Output<'static>) {
    let interface = SPIInterface::new(spi_device, dc);

    let mut display_128: GraphicsMode<_> = ssd1351::builder::Builder::new()
        .with_size(Display128x128)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_interface(interface)
        .into();

    let mut delay = Delay::new();
    display_128.reset(&mut rst, &mut delay).unwrap();
    display_128.init().unwrap();

    let rect_r = Rectangle::new(Point::new(0, 0), Size::new(40, 20))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build());

    rect_r.draw(&mut display_128).unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);

    loop {
        let _state = get_controller_state();

        Text::new("Hello Rust?", Point::new(20, 30), style)
            .draw(&mut display_128)
            .unwrap();
        Timer::after(Duration::from_millis(200)).await;
    }

}