#![feature(int_log)]

use clap::Parser;
use image::{ImageBuffer, Rgb};
use itertools::Itertools;
use palette::{Gradient, LinSrgb};
use rayon::prelude::*;
use rug::{complex::ParseComplexError, float::ParseFloatError, Complex, Float};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Final resolution of the X axis (in pixels)
    #[clap(short = 'x', long, parse(try_from_str=parse_resolution), default_value = "2560x1440")]
    resolution: (u32, u32),

    /// Domain specification for the image
    /// ie. -0.5575,-0.55
    #[clap(short = 'd', long, parse(try_from_str=parse_range))]
    domain: Option<(Float, Float)>,

    /// Range specification for the image
    /// ie. -0.555,-0.5525
    #[clap(short = 'r', long, parse(try_from_str=parse_range))]
    range: Option<(Float, Float)>,

    /// Center the image about this position
    /// (real, imaginary): (1.5, -0.754)
    #[clap(short = 'c', long, parse(try_from_str=parse_point))]
    centered_around: Option<Complex>,

    /// Zoom level about the position
    /// Used with `centered_around`, to provide precision
    /// for rendering (1/(2 ^ zoom))
    #[clap(short = 'z', long)]
    zoom: Option<u32>,

    /// Samples to iterate before deterimining that a
    /// point has converged
    #[clap(short = 't', long, default_value_t = 500)]
    take: usize,

    /// Output file (and format)
    #[clap(short = 'o', long)]
    output: String,

    /// Interval range for Gradient
    /// The gradient shifts in a loop on this interval. Large values
    /// will make closer values less apparent, and smaller values
    /// will make smaller changes more visible
    #[clap(short = 'g', long, default_value = "300")]
    gradient_interval: usize,

    /// Exponential Gradient
    /// Determines if the gradient should be exponential in nature
    #[clap(short = 'e', long)]
    exponential_gradient: bool,
}

fn parse_resolution(resolution: &str) -> Result<(u32, u32), &'static str> {
    let (x, y) = resolution
        .split('x')
        .map(|m| m.parse::<u32>().map_err(|_| "Invalid Resolution"))
        .collect_tuple()
        .expect("Resolution must be in the format 9999x9999");
    Ok((x?, y?))
}

fn parse_point(point: &str) -> Result<Complex, ParseComplexError> {
    let len = num_digits_log2_10(point.split(',').map(|s| {
        // get number of digits needed here as usize
        let mut d: usize = 0;
        if s.contains('-') {
            d += 1;
        }

        if s.contains('e') {
            if let Some(Ok(e)) = s.split('e').last().map(|r| r.parse::<isize>()) {
                d += e.abs() as usize;
            }
        }

        d + s.chars().filter(|c| c.is_digit(10)).count()
    }).max().unwrap());
    let point = Complex::parse(point)?;
    Ok(Complex::with_val(len as u32, point))
}

fn parse_range(range: &str) -> Result<(Float, Float), ParseFloatError> {
    let (begin, end) = range.split(',').collect_tuple().unwrap();
    let precision = num_digits_log2_10(begin.len().max(end.len()));

    let begin = Float::parse(begin)?;
    let end = Float::parse(end)?;

    Ok((
        Float::with_val(precision, begin),
        Float::with_val(precision, end),
    ))
}

fn num_digits_log2_10(d: usize) -> u32 {
    let log2_10: f64 = 10_f64.log2();
    let d = d as f64;
    let d = (d * log2_10).ceil().min(u32::MAX as f64);
    // Always keep 4 bits beyond the requested length
    4 + unsafe { d.to_int_unchecked::<u32>() }
}

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
        z: Complex::with_val(c.prec(), (0_f32, 0_f32)),
        c,
    }
}

fn main() {
    let args = Args::parse();

    let resolution_prec: u32 = (args.resolution.0.max(args.resolution.1).log2() + 1) as u32;

    let take = args.take;
    let prec: u32;
    let x_begin: Float;
    let y_begin: Float;
    let x_step: Float;
    let y_step: Float;

    if let Some((domain_start, domain_end)) = args.domain {
        let (range_start, range_end) = args.range.expect("Domain and Range are both required");
        prec = resolution_prec + domain_start.prec().max(range_start.prec()) + 4;
        x_step = Float::with_val(
            prec,
            Float::with_val(prec, &domain_start - &domain_end)
                / Float::with_val(prec, args.resolution.0),
        );
        y_step = Float::with_val(
            prec,
            Float::with_val(prec, &range_start - &range_end)
                / Float::with_val(prec, args.resolution.1),
        );

        x_begin = Float::with_val(prec, domain_start);
        y_begin = Float::with_val(prec, range_start);
    } else {
        let zoom = args
            .zoom
            .expect("If Domain and Range are not specified, Zoom and Point are required");

        let zoom_p = zoom as f64 * 10_f64.log2();
        let zoom_p = zoom_p.ceil().min(u32::MAX as f64);
        let zoom_p = unsafe { zoom_p.to_int_unchecked::<u32>() };
        prec = zoom_p + 3 + resolution_prec;

        let step = Float::i_pow_u(2, zoom);
        let step = Float::with_val(prec, step);
        let step = step.recip();

        x_step = Float::with_val(prec, &step / args.resolution.0);
        y_step = Float::with_val(prec, &step / args.resolution.1);

        let center = args
            .centered_around
            .expect("If Domain and Range are not specified, Zoom and Point are required");

        println!("Center: ({:?})", center);

        x_begin = center.real() - Float::with_val(prec, &x_step * (args.resolution.0 / 2));
        y_begin = center.imag() - Float::with_val(prec, &y_step * (args.resolution.1 / 2));
    }

    println!("Bits of precision: {}", prec);

    let gradient = if args.exponential_gradient {
        Gradient::with_domain([
            (0_f64, LinSrgb::new(1_f64, 1_f64, 1_f64)),
            (0.5_f64, LinSrgb::new(0.5_f64, 0_f64, 0_f64)),
            (1_f64, LinSrgb::new(1_f64, 0_f64, 0_f64)),
            (2_f64, LinSrgb::new(1_f64, 0.5_f64, 0_f64)),
            (4_f64, LinSrgb::new(0.5_f64, 1_f64, 0.5_f64)),
            (8_f64, LinSrgb::new(0_f64, 1_f64, 1_f64)),
            (16_f64, LinSrgb::new(0_f64, 0.5_f64, 1_f64)),
            (32_f64, LinSrgb::new(0_f64, 0_f64, 1_f64)),
            (64_f64, LinSrgb::new(0.25_f64, 0_f64, 1_f64)),
            (128_f64, LinSrgb::new(1_f64, 1_f64, 1_f64)),
        ])
    } else {
        Gradient::with_domain([
            (0_f64, LinSrgb::new(1_f64, 1_f64, 1_f64)),
            (0.5_f64, LinSrgb::new(0.5_f64, 0_f64, 0_f64)),
            (1.5_f64, LinSrgb::new(1_f64, 0_f64, 0_f64)),
            (2.5_f64, LinSrgb::new(1_f64, 0.5_f64, 0_f64)),
            (3.5_f64, LinSrgb::new(0.5_f64, 1_f64, 0.5_f64)),
            (4.5_f64, LinSrgb::new(0_f64, 1_f64, 1_f64)),
            (5.5_f64, LinSrgb::new(0_f64, 0.5_f64, 1_f64)),
            (6.5_f64, LinSrgb::new(0_f64, 0_f64, 1_f64)),
            (7.5_f64, LinSrgb::new(0.25_f64, 0_f64, 1_f64)),
            (8_f64, LinSrgb::new(1_f64, 1_f64, 1_f64)),
        ])
    };

    let mut img = ImageBuffer::new(args.resolution.0, args.resolution.1);

    for (x, y, p) in (0..args.resolution.0)
        .into_par_iter()
        .flat_map(move |x| (0..args.resolution.1).into_par_iter().map(move |y| (x, y)))
        .filter_map(|(x, y)| {
            let x_val = &x_begin + Float::with_val(prec, x * &x_step);
            let y_val = &y_begin + Float::with_val(prec, y * &y_step);

            let i = square_iter(Complex::with_val(prec, (x_val, y_val)))
                .take(take)
                .count();

            let color = if args.exponential_gradient {
                let pos = i as f64 / take as f64;
                gradient.get(pos * 128_f64)
            } else {
                let pos = i % args.gradient_interval;
                let pos = pos as f64 / args.gradient_interval as f64;
                gradient.get(pos * 8_f64)
            };

            if i < take {
                Some((
                    x,
                    y,
                    Rgb(unsafe {
                        [
                            (color.red * 255_f64).to_int_unchecked::<u8>(),
                            (color.green * 255_f64).to_int_unchecked::<u8>(),
                            (color.blue * 255_f64).to_int_unchecked::<u8>(),
                        ]
                    }),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
    {
        img.put_pixel(x, y, p);
    }

    img.save(&args.output).ok();
    println!("Output saved to: {}", args.output);
}
