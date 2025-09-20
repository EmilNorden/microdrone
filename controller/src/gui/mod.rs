mod assets;
mod battery_indicator;

use alloc::format;
use core::fmt::Debug;
use core::str::FromStr;

use embassy_futures::select::{select3, Either3};
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::Async;
use ssd1351::mode::GraphicsMode;
use ssd1351::prelude::SPIInterface;
use ssd1351::properties::DisplayRotation;
use ssd1351::properties::DisplaySize::Display128x128;
use tinyui::component::Component;

use crate::gui::assets::{
    DRONE_DISCONNECTED_ICON_RAW, DRONE_ICON_RAW, GAMEPAD_CONNECTED_ICON_RAW, GAMEPAD_DISCONNECTED_ICON_RAW,
};
use crate::signal::{BatterySignal, ControllerConnectedSignal, RadioSignal};

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    mut rst: Output<'static>,
    dc: Output<'static>,
    mut battery_signal: BatterySignal,
    mut radio_signal: RadioSignal,
    mut controller_signal: ControllerConnectedSignal,
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

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let mut gamepad_battery_label: Label<'_, _, 15> =
        Label::new("-%", style, Point::new(22, -2), Rgb565::BLACK).unwrap();

    let gamepad_connected_icon = Image::new(&GAMEPAD_CONNECTED_ICON_RAW, Point::new(0, 0));
    let gamepad_disconnected_icon = Image::new(&GAMEPAD_DISCONNECTED_ICON_RAW, Point::new(0, 0));
    let drone_icon = Image::new(&DRONE_ICON_RAW, Point::new(50, 0));
    let drone_disconnected_icon = Image::new(&DRONE_DISCONNECTED_ICON_RAW, Point::new(50, 0));
    loop {
        match select3(
            battery_signal.next_value(),
            controller_signal.next_value(),
            radio_signal.next_value(),
        )
        .await
        {
            Either3::First(battery) => {
                esp_println::println!("DRAWING battery text");
                gamepad_battery_label.set_text(&format!("{}%", battery.level)).unwrap();
                gamepad_battery_label.draw(&mut display).unwrap();
            }
            Either3::Second(connected) => {
                if connected {
                    gamepad_connected_icon.draw(&mut display).unwrap();
                    gamepad_battery_label.set_visible(true);
                } else {
                    gamepad_disconnected_icon.draw(&mut display).unwrap();
                    gamepad_battery_label.set_visible(false);
                }
                gamepad_battery_label.draw(&mut display).unwrap();
            }
            Either3::Third(radio) => {
                if radio.connected {
                    drone_icon.draw(&mut display).unwrap();
                } else {
                    drone_disconnected_icon.draw(&mut display).unwrap();
                }
            }
        }
    }
}

struct Label<'a, C, const N: usize> {
    text: heapless::String<N>,
    style: MonoTextStyle<'a, C>,
    size: Size,
    position: Point,
    clear_area: Rectangle,
    needs_redraw: bool,
    visible: bool,
    background_color: C,
}

impl<'a, C, const N: usize> Label<'a, C, N>
where
    C: PixelColor,
{
    type Err = ();
    fn new(text: &str, style: MonoTextStyle<'a, C>, position: Point, background_color: C) -> Result<Self, Self::Err> {
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
            visible: true,
            background_color,
        })
    }

    fn calculate_size(text: &heapless::String<N>, style: &MonoTextStyle<C>) -> Size {
        let width = style.font.character_size.width * text.len() as u32;
        let height = style.font.character_size.height;

        Size::new(width, height)
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.needs_redraw = true;
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

        if self.visible {
            Text::new(
                self.text.as_str(),
                Point::new(self.position.x, self.position.y + self.size.height as i32 - 1),
                self.style,
            )
            .draw(target)?;
        }

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
            color,
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
