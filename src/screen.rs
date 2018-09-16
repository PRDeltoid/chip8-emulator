//Actual screen will be some multiple of 64 x 32 equal to the size of my "pixels", but actual pixels will be represented directly by a 64 x 32 grid (stored as a linear array)

use piston_window::*;

pub struct Screen {
    x_size: u8,
    y_size: u8,
    pixel_size: f32,
    window: PistonWindow,
}

impl Screen {
    pub fn new(x_size: u8, y_size: u8, pixel_size: f32) -> Screen {
        let height: u32 = x_size as u32 * pixel_size as u32;
        let width: u32 = y_size as u32 * pixel_size as u32;
        println!("Height: {}, Width: {}", height, width);
        let mut screen = Screen {
            x_size, //Amount of pixels on the horizontal scale
            y_size, //Amount of pixels on the vertical scale
            pixel_size, //Size of each pixel in pixels
            window: WindowSettings::new(
                "test",
                [height, width]
            )
            .exit_on_esc(true)
            .build()
            .unwrap()
        };

        screen.window.set_lazy(true);
        screen
    }

    pub fn clear(&mut self) {
        let event = self.window.next().unwrap();
        self.window.draw_2d(&event, |_context, graphics| {
            clear(color::BLACK, graphics);
        });
    }

    pub fn draw(&mut self, event: Event, screen: &[u8; 64 * 32]) {
        let pixel_size = self.pixel_size as f64;
        let y_size = self.y_size as usize;
        let x_size = self.x_size as usize;

        //BUG: Code gets stuck in this loop when drawing
        while let Some(e) = self.window.next() {
            //let e = self.window.next().unwrap();
            self.window.draw_2d(&e, |c, g| {

                //Step over each x "pixel"
                for x in 0..x_size as usize {
                    //Step over each y "pixel" for each x above
                    for y in 0..y_size as usize {
                        //If the screen contains a 1 at the current pixel...
                        if screen[x + (y * x_size as usize)] == 1 {
                            let x_pos = x as f64 * pixel_size;
                            let y_pos = y as f64 * pixel_size;
                            println!("Drawing rect at x:{}, y:{}", x_pos, y_pos);
                            Rectangle::new([1.0, 1.0, 1.0, 1.0])
                                .draw([x_pos, y_pos, pixel_size, pixel_size], &c.draw_state, c.transform, g)
                        }
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use Screen;
    #[test]
    fn test_screen() {
        let mut screen = Screen::new(64, 32, 8.0);
        let mut screen_buff: [u8; 64 * 32] = [0; 64 * 32];

        screen_buff[61] = 1;

        screen.draw(&screen_buff);
    }
}