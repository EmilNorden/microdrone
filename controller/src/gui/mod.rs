mod battery_indicator;

use crate::input::{get_controller_state, get_input_state};
use alloc::format;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::fmt::Debug;
use core::str::FromStr;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::ascii::FONT_6X10,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::{delay::Delay, gpio::Output, spi::master::Spi, Async};
use ssd1351::{
    mode::GraphicsMode, prelude::SPIInterface, properties::DisplayRotation,
    properties::DisplaySize::Display128x128,
};
use tinyui::component::Component;
use tinyui::context::Context;
use tinyui::frame::Frame;

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    mut rst: Output<'static>,
    dc: Output<'static>,
) {
    let interface = SPIInterface::new(spi_device, dc);

    esp_println::println!("Creating display");
    let mut display: GraphicsMode<_> = ssd1351::builder::Builder::new()
        .with_size(Display128x128)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_interface(interface)
        .into();

    let mut delay = Delay::new();
    display.reset(&mut rst, &mut delay).unwrap();
    display.init().unwrap();

    /*let rect_r = Rectangle::new(Point::new(0, 0), Size::new(40, 20))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build());

    rect_r.draw(&mut display).unwrap();*/

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let mut ctrl_status_label: Label<'_, _, 15> =
        Label::new(
            "Not connected",
            style,
            Point::new(10, 10),
            Rgb565::BLACK,
        )
        .unwrap();

    let mut throttle_indicator = Rc::new(RefCell::new(ThrottleIndicator::new(Rgb565::GREEN)));

    /*let mut ctx = Context::new(display);
    let mut frame = Frame::new(Rgb565::BLACK);
    frame.add_component(ctrl_status_label.clone());
    //frame.add_component(throttle_indicator);
    ctx.set_frame(frame);*/

    loop {
        let input_state = get_input_state();
        let controller_state = get_controller_state();

        if controller_state.connected {
            /*let mut mut_borrow = ctrl_status_label.borrow_mut();
            mut_borrow
                .set_text(&format!("{}%", controller_state.battery))
                .unwrap();*/
            //draw_battery_level(&mut display, style, controller_state.battery);

            ctrl_status_label
                .set_text(&format!("{}%", controller_state.battery))
                .unwrap();

        } else {
            /*let mut mut_borrow = ctrl_status_label.borrow_mut();
            mut_borrow.set_text("Not connected").unwrap();*/

            ctrl_status_label
                .set_text("Not connected")
                .unwrap();
        }

        //ctx.draw().unwrap();
        ctrl_status_label.draw(&mut display).unwrap();
        Timer::after(Duration::from_millis(200)).await;
    }
}

struct Label<'a, C, const N: usize> {
    text: heapless::String<N>,
    style: MonoTextStyle<'a, C>,
    size: Size,
    position: Point,
    clear_area: Rectangle,
    needs_redraw: bool,
    background_color: C,
}

impl<'a, C, const N: usize> Label<'a, C, N>
where
    C: PixelColor,
{
    type Err = ();
    fn new(
        text: &str,
        style: MonoTextStyle<'a, C>,
        position: Point,
        background_color: C,
    ) -> Result<Self, Self::Err> {
        let text = heapless::String::from_str(text)?;
        let size = Self::calculate_size(&text, &style);
        let clear_area = Rectangle::new(position, size);

        Ok(Self {
            text,
            style,
            size,
            position,
            clear_area,
            needs_redraw: true,
            background_color,
        })
    }

    fn calculate_size(text: &heapless::String<N>, style: &MonoTextStyle<C>) -> Size {
        let width = style.font.character_size.width * text.len() as u32;
        let height = style.font.character_size.height;

        esp_println::println!(
            "Text length: {}. Char size: {} x {}. Calc size  {} x {}",
            text.len(),
            style.font.character_size.width,
            style.font.character_size.height,
            width,
            height
        );
        Size::new(width, height)
    }

    pub fn set_text(&mut self, text: &str) -> Result<(), Self::Err> {
        if text == self.text {
            return Ok(());
        }
        esp_println::println!("Setting text to {}", text);

        self.text = heapless::String::from_str(text)?;
        self.size = Self::calculate_size(&self.text, &self.style);
        self.needs_redraw = true;

        Ok(())
    }

    fn draw<D>(&mut self, target: &mut D) -> Result<(), <D as DrawTarget>::Error>
    where
        C: PixelColor,
        D: DrawTarget<Color = C>,
    {
        if !self.needs_redraw {
            return Ok(());
        }

        self.clear_area
            .into_styled(PrimitiveStyleBuilder::new().fill_color(self.background_color).build())
            .draw(target)?;

        Text::new(
            self.text.as_str(),
            Point::new(
                self.position.x,
                self.position.y + self.size.height as i32 - 1,
            ),
            self.style,
        )
            .draw(target)?;

        self.needs_redraw = false;

        Ok(())
    }
}

struct ThrottleIndicator<C> {
    needs_redraw: bool,
    throttle: u8,
    color: C,
}

impl<C> ThrottleIndicator<C>
where
    C: PixelColor,
{
    const HEIGHT: u32 = 5;

    pub fn new(color: C) -> Self {
        Self {
            needs_redraw: false,
            throttle: 0,
            color
        }
    }
    pub fn set_throttle(&mut self, throttle: u8) {
        if self.throttle != throttle {
            self.throttle = throttle;
            self.needs_redraw = true;
        }
    }

}

impl<C> Component<C> for ThrottleIndicator<C>
where
    C: PixelColor,
{
    fn size(&self) -> Size {
        Size::new(128, Self::HEIGHT)
    }

    fn position(&self) -> Point {
        Point::new(0, 128 - Self::HEIGHT as i32)
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn draw<D>(&mut self, target: &mut D) -> Result<(), <D as DrawTarget>::Error>
    where
        C: PixelColor,
        D: DrawTarget<Color = C>,
    {
        let rect = Rectangle::new(self.position(), self.size())
            .into_styled(PrimitiveStyleBuilder::new().fill_color(self.color).build());

        rect.draw(target)?;

        Ok(())
    }
}
