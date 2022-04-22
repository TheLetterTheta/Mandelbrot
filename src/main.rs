use image::{ImageBuffer, Rgb};
use rug::Float;

struct SquaresComplex {
    z_real: Float,
    z_imaginary: Float,
    c_real: Float,
    c_imaginary: Float,
    prec: u32,
}

impl Iterator for SquaresComplex {
    type Item = (Float, Float);

    fn next(&mut self) -> Option<Self::Item> {
        let (new_real, new_imaginary) = {
            let curr_real = &self.z_real;
            let c_imaginary = &self.c_imaginary;
            let c_real = &self.c_real;
            let curr_imaginary = &self.z_imaginary;

            let new_real = Float::with_val(
                self.prec,
                Float::with_val(
                    self.prec,
                    (curr_real * curr_real) - (curr_imaginary * curr_imaginary),
                ) + c_real,
            );
            let new_imaginary = Float::with_val(
                self.prec,
                Float::with_val(4, 2_u8) * curr_real * curr_imaginary + c_imaginary,
            );

            (new_real, new_imaginary)
        };

        self.z_real = new_real.clone();
        self.z_imaginary = new_imaginary.clone();

        if Float::with_val(
            self.prec,
            (&self.z_real * &self.z_real) + (&self.z_imaginary * &self.z_imaginary),
        ) > Float::with_val(4, 4_u8)
        {
            None
        } else {
            Some((new_real, new_imaginary))
        }
    }
}

fn square_iter(prec: u32, c_real: Float, c_imaginary: Float) -> SquaresComplex {
    SquaresComplex {
        z_real: Float::with_val(4, 0),
        z_imaginary: Float::with_val(4, 0),
        c_real,
        c_imaginary,
        prec,
    }
}

fn main() {
    const DOMAIN_PREC: u32 = 3;
    const RESOLUTION_PREC: u32 = 12;
    const TAKE: usize = 1000;

    const PREC: u32 = DOMAIN_PREC + RESOLUTION_PREC + 10;

    let domain: [Float; 2] = [Float::with_val(PREC, -1.5_f32), Float::with_val(PREC, 0.5_f32)];
    let range: [Float; 2] = [Float::with_val(PREC, -1_f32), Float::with_val(PREC, 1_f32)];

    const X_RESOLUTION: u32 = 1440;
    const Y_RESOLUTION: u32 = 1080;

    let x_step = Float::with_val(
        PREC,
        Float::with_val(PREC, &domain[1] - &domain[0]) / Float::with_val(PREC, X_RESOLUTION),
    );
    let y_step = Float::with_val(
        PREC,
        Float::with_val(PREC, &range[1] - &range[0]) / Float::with_val(PREC, Y_RESOLUTION),
    );

    let img = ImageBuffer::from_fn(X_RESOLUTION, Y_RESOLUTION, |x, y| {
        let x_val = &domain[0] + Float::with_val(PREC, x * &x_step);
        let y_val = &range[0] + Float::with_val(PREC, y * &y_step);
        let i = square_iter(PREC, x_val, y_val).take(TAKE).count();

        if i == TAKE {
            Rgb([0, 0, 0])
        } else {
            let pos = ((i / 1000) * 255) as u8;
            Rgb([pos / 3, 255, 0])
        }
    });

    img.save("Test 2.png").ok();
}
