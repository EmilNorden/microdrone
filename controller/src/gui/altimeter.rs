use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::Point;

pub struct Altimeter<'a, C> {
    position: Point,
    style: MonoTextStyle<'a, C>,
    altitude: u8,
    background_color: C,
}

impl<'a, C> Altimeter<'a, C>
where
    C: PixelColor,
{
    pub fn new(position: Point, style: MonoTextStyle<'a, C>, background_color: C) -> Self {
        Self {
            position,
            style,
            altitude: 0,
            background_color,
        }
    }

    pub fn draw<D>(&mut self, target: &mut D) -> Result<(), <D as DrawTarget>::Error>
    where
        C: PixelColor,
        D: DrawTarget<Color = C>,
    {
        Ok(())
    }
}
