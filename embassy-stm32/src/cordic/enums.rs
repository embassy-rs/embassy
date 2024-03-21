/// CORDIC function
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[derive(Debug, Clone, Copy, Default)]
pub enum Precision {
    Iters4 = 1,
    Iters8,
    Iters12,
    Iters16,
    Iters20,
    #[default]
    Iters24, // this value is recommended by Reference Manual
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
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Scale {
    #[default]
    Arg1Res1 = 0,
    Arg1o2Res2,
    Arg1o4Res4,
    Arg1o8Res8,
    Arg1o16Res16,
    Arg1o32Res32,
    Arg1o64Res64,
    Arg1o128Res128,
}

/// CORDIC argument/result register access count
#[allow(missing_docs)]
#[derive(Clone, Copy, Default)]
pub enum AccessCount {
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
