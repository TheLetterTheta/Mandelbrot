use image::{ImageBuffer, Rgb};
use palette::{Gradient, LinSrgb};
use rayon::prelude::*;
use rug::{Complex, Float};

const DOMAIN_PREC: u32 = 20;
const RESOLUTION_PREC: u32 = 12;
const TAKE: usize = 100;

const PREC: u32 = DOMAIN_PREC + RESOLUTION_PREC + 10;

struct SquaresComplex {
    z: Complex,
    c: Complex,
}

impl Iterator for SquaresComplex {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.z.square_mut();
        self.z += &self.c;

        let dist = Float::with_val(5, self.z.abs_ref());
        if dist > Float::with_val(5, 4_f32) {
            None
        } else {
            Some(())
        }
    }
}

fn square_iter(c: Complex) -> SquaresComplex {
    SquaresComplex {
        z: Complex::with_val(PREC, (0_f32, 0_f32)),
        c,
    }
}

fn main() {
    let domain: [Float; 2] = [
        Float::with_val(PREC, -0.65_f32),
        Float::with_val(PREC, -0.55_f32),
    ];
    let range: [Float; 2] = [
        Float::with_val(PREC, -0.55_f32),
        Float::with_val(PREC, -0.65_f32),
    ];

    const X_RESOLUTION: u32 = 2560;
    const Y_RESOLUTION: u32 = 1440;

    let gradient = Gradient::new([
        LinSrgb::new(1_f64, 0.5_f64, 0.5_f64),
        LinSrgb::new(0.5_f64, 1_f64, 0.5_f64),
        LinSrgb::new(0.5_f64, 0.5_f64, 1_f64),
    ]);

    let x_step = Float::with_val(
        PREC,
        Float::with_val(PREC, &domain[1] - &domain[0]) / Float::with_val(PREC, X_RESOLUTION),
    );
    let y_step = Float::with_val(
        PREC,
        Float::with_val(PREC, &range[1] - &range[0]) / Float::with_val(PREC, Y_RESOLUTION),
    );

    let mut img = ImageBuffer::new(X_RESOLUTION, Y_RESOLUTION);

    for (x, y, p) in (0..X_RESOLUTION)
        .into_par_iter()
        .flat_map(move |x| (0..Y_RESOLUTION).into_par_iter().map(move |y| (x, y)))
        .map(|(x, y)| {
            let x_val = &domain[0] + Float::with_val(PREC, x * &x_step);
            let y_val = &range[0] + Float::with_val(PREC, y * &y_step);

            let i = square_iter(Complex::with_val(PREC, (x_val, y_val)))
                .take(TAKE)
                .count();

            if i == TAKE {
                (x, y, Rgb([0, 0, 0]))
            } else {
                let pos = i as f64 / TAKE as f64;
                let color = gradient.get(pos);

                (
                    x,
                    y,
                    Rgb(unsafe {
                        [
                            (color.red * 255_f64).to_int_unchecked::<u8>(),
                            (color.green * 255_f64).to_int_unchecked::<u8>(),
                            (color.blue * 255_f64).to_int_unchecked::<u8>(),
                        ]
                    }),
                )
            }
        })
        .collect::<Vec<_>>()
    {
        img.put_pixel(x, y, p);
    }

    img.save("Test 3.png").ok();
}
