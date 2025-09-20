use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::Rgb565;

pub struct Graphic<const N: usize> {
    pub width: u32,
    pub height: u32,
    pub data: [u8; N],
}

impl<const N: usize> Graphic<N> {
    pub const fn new(width: u32, height: u32, data: [u8; N]) -> Self {
        Self { width, height, data }
    }
}

const GAMEPAD_CONNECTED_GRAPHICS: Graphic<320> =
    Graphic::new(16, 10, *include_bytes!(concat!(env!("OUT_DIR"), "/gamepad.raw")));
pub const GAMEPAD_CONNECTED_ICON_RAW: ImageRaw<Rgb565> =
    ImageRaw::<Rgb565>::new(&GAMEPAD_CONNECTED_GRAPHICS.data, GAMEPAD_CONNECTED_GRAPHICS.width);

const GAMEPAD_DISCONNECTED_GRAPHICS: Graphic<320> = Graphic::new(
    16,
    10,
    *include_bytes!(concat!(env!("OUT_DIR"), "/gamepad_disconnect.raw")),
);
pub const GAMEPAD_DISCONNECTED_ICON_RAW: ImageRaw<Rgb565> =
    ImageRaw::<Rgb565>::new(&GAMEPAD_DISCONNECTED_GRAPHICS.data, GAMEPAD_DISCONNECTED_GRAPHICS.width);

const DRONE_GRAPHICS: Graphic<320> = Graphic::new(16, 10, *include_bytes!(concat!(env!("OUT_DIR"), "/drone.raw")));

pub const DRONE_ICON_RAW: ImageRaw<Rgb565> = ImageRaw::<Rgb565>::new(&DRONE_GRAPHICS.data, DRONE_GRAPHICS.width);

const DRONE_DISCONNECTED_GRAPHICS: Graphic<320> = Graphic::new(
    16,
    10,
    *include_bytes!(concat!(env!("OUT_DIR"), "/drone_disconnect.raw")),
);

pub const DRONE_DISCONNECTED_ICON_RAW: ImageRaw<Rgb565> =
    ImageRaw::<Rgb565>::new(&DRONE_DISCONNECTED_GRAPHICS.data, DRONE_DISCONNECTED_GRAPHICS.width);
