/// CORDIC function
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Function {
    Cos = 0,
    Sin,
    Phase,
    Modulus,
    Arctan,
    Cosh,
    Sinh,
    Arctanh,
    Ln,
    Sqrt,
}

/// CORDIC precision
#[allow(missing_docs)]
#[derive(Clone, Copy, Default)]
pub enum Precision {
    Iters4 = 1,
    Iters8,
    Iters12,
    Iters16,
    Iters20,
    #[default]
    Iters24, // this value is recomended by Reference Manual
    Iters28,
    Iters32,
    Iters36,
    Iters40,
    Iters44,
    Iters48,
    Iters52,
    Iters56,
    Iters60,
}

/// CORDIC scale
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Default, PartialEq)]
pub enum Scale {
    #[default]
    A1_R1 = 0,
    A1o2_R2,
    A1o4_R4,
    A1o8_R8,
    A1o16_R16,
    A1o32_R32,
    A1o64_R64,
    A1o128_R128,
}

/// CORDIC argument/result count
#[allow(missing_docs)]
#[derive(Clone, Copy, Default)]
pub enum Count {
    #[default]
    One,
    Two,
}

/// CORDIC argument/result data width
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Width {
    Bits32,
    Bits16,
}
