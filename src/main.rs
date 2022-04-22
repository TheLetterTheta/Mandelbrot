use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::Ratio;
use num_traits::{identities::Zero, sign::abs};
use std::ops::{Mul, Sub};

struct SquaresComplex {
    z_real: Ratio<BigInt>,
    z_imaginary: Ratio<BigInt>,
    c_real: Ratio<BigInt>,
    c_imaginary: Ratio<BigInt>,
}

impl Iterator for SquaresComplex {
    type Item = (Ratio<BigInt>, Ratio<BigInt>);

    fn next(&mut self) -> Option<Self::Item> {
        let curr_real = self.z_real.clone();
        let curr_imaginary = self.z_imaginary.clone();

        self.z_real =
            (&self.z_real * &self.z_real) - (&self.z_imaginary * &self.z_imaginary) + &self.c_real;
        self.z_imaginary = (Ratio::from_integer(BigInt::from(2_u8))
            * &self.z_real
            * &self.z_real
            * &self.z_imaginary)
            + &self.c_imaginary;

        if (&self.z_real * &self.z_real) + (&self.z_imaginary * &self.z_imaginary)
            > Ratio::from_integer(BigInt::from(4_u8))
        {
            None
        } else {
            Some((curr_real, curr_imaginary))
        }
    }
}

fn square_iter(c_real: Ratio<BigInt>, c_imaginary: Ratio<BigInt>) -> SquaresComplex {
    SquaresComplex {
        z_real: Ratio::from_integer(BigInt::zero()),
        z_imaginary: Ratio::from_integer(BigInt::zero()),
        c_real,
        c_imaginary,
    }
}

fn main() {
    let start = Ratio::new(BigInt::from(-3_i8), BigInt::from(5_i8));
    let end = Ratio::from_integer(BigInt::zero());

    for i in square_iter(start, end) {
        println!("({}, {})", i.0.to_string(), i.1.to_string());
    }
}
