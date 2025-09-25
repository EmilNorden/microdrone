mod altimeter;
use uom::num_traits::Float;
mod assets;
mod battery_indicator;
mod label;

use alloc::format;
use core::fmt::Debug;
use core::str::FromStr;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_futures::select::{select6, Either6};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::ascii::FONT_8X13_BOLD;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;
use fc_common::SignalBase;
use ssd1351::mode::GraphicsMode;
use ssd1351::prelude::SPIInterface;
use ssd1351::properties::DisplayRotation;
use ssd1351::properties::DisplaySize::Display128x128;
use tinyui::component::Component;
use uom::num_traits::Euclid;

use crate::gui::assets::{
    DRONE_DISCONNECTED_ICON_RAW, DRONE_ICON_RAW, GAMEPAD_CONNECTED_ICON_RAW, GAMEPAD_DISCONNECTED_ICON_RAW,
};
use crate::gui::label::Label;
use crate::signal::{
    BatterySignal, ControllerConnectedSignal, DroneAltitudeSignal, DroneBatteryLevelSignal, RadioLinkQualitySignal,
    RadioSignal,
};

#[embassy_executor::task]
pub async fn run(
    spi_device: SpiDevice<'static, NoopRawMutex, Spi<'static, Blocking>, Output<'static>>,
    mut rst: Output<'static>,
    dc: Output<'static>,
    mut battery_signal: BatterySignal,
    mut radio_signal: RadioSignal,
    mut controller_signal: ControllerConnectedSignal,
    mut drone_battery_signal: DroneBatteryLevelSignal,
    mut drone_altitude_signal: DroneAltitudeSignal,
    mut radio_link_quality_signal: RadioLinkQualitySignal,
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

    let style = MonoTextStyle::new(&FONT_8X13_BOLD, Rgb565::WHITE);
    let mut gamepad_battery_label: Label<'_, _, 15> =
        Label::new("-%", style, Point::new(22, -3), Rgb565::BLACK).unwrap();
    let mut drone_battery_label: Label<'_, _, 15> = Label::new("-%", style, Point::new(92, -3), Rgb565::BLACK).unwrap();
    let mut altitude_label: Label<'_, _, 15> = Label::new("-%", style, Point::new(0, 40), Rgb565::BLACK).unwrap();
    let mut quality_label: Label<'_, _, 15> = Label::new("-%", style, Point::new(0, 70), Rgb565::BLACK).unwrap();

    let gamepad_connected_icon = Image::new(&GAMEPAD_CONNECTED_ICON_RAW, Point::new(0, 0));
    let gamepad_disconnected_icon = Image::new(&GAMEPAD_DISCONNECTED_ICON_RAW, Point::new(0, 0));
    let drone_icon = Image::new(&DRONE_ICON_RAW, Point::new(70, 0));
    let drone_disconnected_icon = Image::new(&DRONE_DISCONNECTED_ICON_RAW, Point::new(70, 0));
    loop {
        match select6(
            battery_signal.next_value(),
            controller_signal.next_value(),
            radio_signal.next_value(),
            core::future::pending::<()>(),
            //drone_battery_signal.next_value(),
            drone_altitude_signal.next_value(),
            radio_link_quality_signal.next_value(),
        )
        .await
        {
            Either6::First(battery) => {
                esp_println::println!("DRAWING battery text");
                gamepad_battery_label.set_text(&format!("{}%", battery.level)).unwrap();
                gamepad_battery_label.draw(&mut display).unwrap();
            }
            Either6::Second(connected) => {
                if connected {
                    gamepad_connected_icon.draw(&mut display).unwrap();
                    gamepad_battery_label.set_visible(true);
                } else {
                    gamepad_disconnected_icon.draw(&mut display).unwrap();
                    gamepad_battery_label.set_visible(false);
                }
                gamepad_battery_label.draw(&mut display).unwrap();
            }
            Either6::Third(radio) => {
                if radio.connected {
                    drone_icon.draw(&mut display).unwrap();
                    drone_battery_label.set_visible(true);
                } else {
                    drone_disconnected_icon.draw(&mut display).unwrap();
                    drone_battery_label.set_visible(false);
                }
                drone_battery_label.draw(&mut display).unwrap();
            }
            Either6::Fourth(level) => {
                drone_battery_label.set_text(&format!("{}%", 0)).unwrap();
                drone_battery_label.draw(&mut display).unwrap();
            }
            Either6::Fifth(altitude) => {
                let (meters, quarters) = altitude.div_rem_euclid(&4);
                altitude_label
                    .set_text(&format!("Alt: {}.{}m", meters, quarters * 25))
                    .unwrap();
                altitude_label.draw(&mut display).unwrap();
            }
            Either6::Sixth(quality) => {
                quality_label
                    .set_text(&format!("Link: {}%", (quality * 100.0).round()))
                    .unwrap();
                quality_label.draw(&mut display).unwrap();
            }
        }
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
