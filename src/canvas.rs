use crate::color::Color;

/// A 2D canvas. The (0, 0) coordinate is at the top left.
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::new(0.0, 0.0, 0.0); width * height],
        }
    }

    /// Writes the color at some pixel. If the specified coordinate is beyond the initialized size,
    /// it is ignored.
    pub fn write_to(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        self.pixels[idx] = color;
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Color {
        let idx = y * self.width + x;
        self.pixels[idx]
    }

    /// Get a mutable reference to the pixels of this canvas.
    pub fn pixels_mut(&mut self) -> &mut Vec<Color> {
        &mut self.pixels
    }

    /// Exports the current canvas as a PPM format string.
    pub fn to_ppm(&self) -> String {
        let mut hdr = format!("P3\n{} {}\n255\n", self.width, self.height);
        let pxs = self
            .pixels
            .chunks(self.width)
            .flat_map(|row| {
                let nums: Vec<String> = row
                    .into_iter()
                    .flat_map(|color| {
                        let s = color.to_string();
                        let nums: Vec<String> =
                            s.split_whitespace().map(|x| x.to_owned()).collect();
                        nums
                    })
                    .collect();
                let mut res: Vec<String> = vec![];
                let mut tmp_str = String::new();
                let mut n = 0;
                for num in nums {
                    let k = num.chars().count();
                    if n + k > 70 {
                        // push the current tmp_str into result and create a new one
                        tmp_str.pop(); // remove the trailing whitespace
                        res.push(tmp_str);
                        tmp_str = String::new();
                        n = 0;
                    }
                    tmp_str.push_str(&num);
                    tmp_str.push(' ');
                    n += k + 1;
                }
                // flush the last tmp_str
                tmp_str.pop();
                res.push(tmp_str);
                res
            })
            .fold(String::new(), |mut accum, s| {
                accum.push_str(&s);
                accum.push('\n');
                accum
            });
        hdr.push_str(&pxs);
        hdr
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::Canvas;
    use crate::color::Color;

    #[test]
    fn create_canvas() {
        let c = Canvas::new(10, 20);
        assert_eq!(c.width, 10);
        assert_eq!(c.height, 20);
        assert_eq!(c.pixels.len(), 10 * 20); // ensure that pixels are actually initialized
        let black = Color::new(0.0, 0.0, 0.0);
        for pixel in c.pixels {
            assert_eq!(pixel, black);
        }
    }

    #[test]
    fn write_pixel() {
        let mut c = Canvas::new(10, 20);
        let red = Color::new(1.0, 0.0, 0.0);
        c.write_to(2, 3, red);
        assert_eq!(c.pixel_at(2, 3), red);
    }

    #[test]
    fn to_ppm() {
        let c = Canvas::new(5, 3);
        let want = "P3\n5 3\n255";
        let got = c.to_ppm();
        assert!(want.lines().eq(got.lines().take(3)));

        let mut c = Canvas::new(5, 3);
        let c1 = Color::new(1.5, 0.0, 0.0);
        let c2 = Color::new(0.0, 0.5, 0.0);
        let c3 = Color::new(-0.5, 0.0, 1.0);
        c.write_to(0, 0, c1);
        c.write_to(2, 1, c2);
        c.write_to(4, 2, c3);
        let want = vec![
            "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0",
            "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0",
            "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255",
        ];
        let got = c.to_ppm();
        let got: Vec<&str> = got.lines().skip(3).take(3).collect();
        assert_eq!(want.len(), got.len());
        for (want, got) in want.into_iter().zip(got) {
            assert_eq!(want, got);
        }
    }

    #[test]
    fn to_ppm_long_lines() {
        let mut c = Canvas::new(10, 2);
        for px in c.pixels.iter_mut() {
            *px = Color::new(1.0, 0.8, 0.6);
        }
        let want = vec![
            "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204",
            "153 255 204 153 255 204 153 255 204 153 255 204 153",
            "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204",
            "153 255 204 153 255 204 153 255 204 153 255 204 153",
        ];
        let got = c.to_ppm();
        let got: Vec<&str> = got.lines().skip(3).take(4).collect();
        assert_eq!(want.len(), got.len());
        for (want, got) in want.into_iter().zip(got) {
            assert_eq!(want, got);
        }
    }

    #[test]
    fn ppm_ends_with_newline() {
        let c = Canvas::new(5, 3);
        let mut ppm = c.to_ppm();
        assert_eq!(ppm.pop().unwrap(), '\n');
    }
}
