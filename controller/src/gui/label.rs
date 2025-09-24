use core::str::FromStr;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;

pub struct Label<'a, C, const N: usize> {
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
    pub type Err = ();
    pub fn new(
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
        self.text = heapless::String::from_str(text)?;
        self.size = Self::calculate_size(&self.text, &self.style);
        self.needs_redraw = true;

        Ok(())
    }

    pub fn draw<D>(&mut self, target: &mut D) -> Result<(), <D as DrawTarget>::Error>
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

            self.clear_area = Rectangle::new(self.position, self.size);
        }

        self.needs_redraw = false;

        Ok(())
    }
}
