//Actual screen will be some multiple of 64 x 32 equal to the size of my "pixels", but actual pixels will be represented directly by a 64 x 32 grid (stored as a linear array)
pub struct Screen {
    _pixel_size: u8,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            _pixel_size: 8, //Each "pixel" is a  white 8x8 pixel square
        }
    }

    pub fn draw(&self, screen: &[u8; 64 * 32]) {
        for x in 0..64 {
            for y in 0..32 {
                //If the screen contains a 1 at the current pixel...
                if screen[x + (y * 32)] == 1 {
                    //Draw an 8x8 white square at coord (x*pixel_size,y*pixel_size)
                }
            }
        }
    }
}