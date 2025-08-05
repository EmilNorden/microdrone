use alloc::format;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb555, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

pub struct BatteryIndicator<'a, C> {
    percentage: u8,
    style: MonoTextStyle<'a, C>,
    position: Point,
}

impl<'a, C> BatteryIndicator<'a, C>
where
    C: PixelColor,
{
    pub fn new(percentage: u8, style: MonoTextStyle<'a, C>, position: Point) -> Self {
        Self { percentage, style, position }
    }

    pub fn draw<D>(&self, target: &mut D) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = C>
    {
        Text::new(&format!("{}%", self.percentage), self.position, self.style)
            .draw(target)
    }

}
/*
impl<C> Component<C> for BatteryIndicator<'_, C>
{
    fn size(&self) -> Size {
        let width = self.style.font.character_size.width * 4;
        let height = self.style.font.character_size.height;
        Size::new(width, height)
    }

    fn position(&self) -> Point {
        self.position
    }

    fn draw<D>(&self, target: &mut D) -> Result<Point, <D as DrawTarget>::Error>
    where
        C: PixelColor,
        D: DrawTarget<Color=C>
    {
        Text::new(&format!("{}%", self.percentage), self.position, self.style)
            .draw(target)
    }
}*/