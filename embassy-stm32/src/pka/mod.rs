//! PKA Hardware accelerator

use core::future::poll_fn;
use core::iter;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc};

// List of RAM addresses for data. Since embassy implementation of RAM access already moves base address to
// 0x0400 and multiplies offset by 4 for 32-bit access, we have to take address from reference manual, subtract 0x0400
// and divide it by 4 to get the correct offset.
// Allowed address space is: 0x0400-0x11F4
const PKA_RAM_OFFSET: usize = 0x0400;

const MODULUS_LENGTH_ADDR: usize = (0x0400 - PKA_RAM_OFFSET) / 4;
const EXPONENT_LENGTH_ADDR: usize = (0x0400 - PKA_RAM_OFFSET) / 4;
const OPERAND_LENGTH_ADDR: usize = (0x0404 - PKA_RAM_OFFSET) / 4;
const ARITMETIC_OPERAND1_ADDR: usize = (0x08B4 - PKA_RAM_OFFSET) / 4;
const ARITMETIC_OPERAND2_ADDR: usize = (0x0A44 - PKA_RAM_OFFSET) / 4;
const ARITMETIC_OPERAND3_ADDR: usize = (0x0D5C - PKA_RAM_OFFSET) / 4;
const EXPONENT_IN_ADDR: usize = (0x0BD0 - PKA_RAM_OFFSET) / 4;

const MONTGOMERY_PARAM_ADDR: usize = (0x0594 - PKA_RAM_OFFSET) / 4;
const ARITMETIC_RESULT_ADDR: usize = (0x0BD0 - PKA_RAM_OFFSET) / 4;
const EXPONENT_RESULT_ADDR: usize = (0x0724 - PKA_RAM_OFFSET) / 4;

const INPUT_OPERAND_MAX_BIT_SIZE: usize = 3136;
const INPUT_OPERAND_MAX_SIZE: usize = INPUT_OPERAND_MAX_BIT_SIZE / 8;

static PKA_WAKER: AtomicWaker = AtomicWaker::new();

/// PKA Operation Mode
#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum PkaOperationMode {
    /// Montgomery Parameter Computation then Modular Exponentiation
    ModularExponentiation = 0b000000,
    /// Montgomery Parameter Computation Only
    MontgomeryParameterComputationOnly = 0b000001,
    /// Modular Exponentiation Only. (Montgomery parameter must be loaded first)
    ModularExponentiationFastMode = 0b000010,
    /// Montgomery Parameter Computation then ECC Scalar Multiplication
    MontgomeryParameterComputationThenEccScalarMultiplication = 0b100000,
    /// ECC Scalar Multiplication Only. (Montgomery parameter must be loaded first)
    EccScalarMultiplicationOnly = 0b100010,
    /// EDCSA Sign
    EcdsaSign = 0b100100,
    /// EDCSA Verification
    EcdsaVerification = 0b100110,
    /// Point On Elliptic Curve FP Check
    PointOnEllipticCurveFpCheck = 0b101000,
    /// RSA CRT Exponentiation
    RsaCrtExponentiation = 0b000111,
    /// Modular Inversion
    ModularInversion = 0b001000,
    /// Arithmetic Addition
    ArithmeticAddition = 0b001001,
    /// Arithmetic Subtraction
    ArithmeticSubtraction = 0b001010,
    /// Arithmetic Multiplication
    ArithmeticMultiplication = 0b001011,
    /// Arithmetic Comparison
    ArithmeticComparison = 0b001100,
    /// Modular Reduction
    ModularReduction = 0b001101,
    /// Modular Addition
    ModularAddition = 0b001110,
    /// Modular Subtraction
    ModularSubtraction = 0b001111,
    /// Montgomery Multiplication
    MontgomeryMultiplication = 0b010000,
}

/// PKA SealedInstance trait.
///
/// Allows access to PAC exclusively for `PKA` module.
trait SealedInstance {
    fn regs() -> pac::pka::Pka;
}

/// PKA interrupt
pub struct InterruptHandler<T: Instance> {
    __phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let procendf = T::regs().sr().read().procendf();
        let addr_err = T::regs().sr().read().addrerrf();
        let ram_err = T::regs().sr().read().ramerrf();

        if procendf {
            T::regs().cr().modify(|w| w.set_procendie(false));
        }

        if addr_err {
            T::regs().cr().modify(|w| w.set_addrerrie(false));
        }

        if ram_err {
            T::regs().cr().modify(|w| w.set_ramerrie(false));
        }

        PKA_WAKER.wake();
    }
}

/// PKA accelerator
pub struct Pka<'d, T> {
    _peripheral: PeripheralRef<'d, T>,
}

/// PKA error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PkaError {
    /// Invalid input parameter
    InvalidInputParameter,
    /// Not enough space to store result
    ResultBufferTooShort,
    /// Operation not supported
    NotSupported,
}

/// PKA Result type alias
pub type PkaResult<T> = Result<T, PkaError>;

impl<'d, T: Instance> Pka<'d, T> {
    /// Create a new PKA accelerator
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        into_ref!(peri);
        let instance = Self { _peripheral: peri };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Disable PKA accelerator
    pub fn disable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(false));
    }

    /// Enable PKA accelerator
    pub fn enable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(true));
    }

    async fn start(&mut self) {
        T::regs().cr().modify(|w| w.set_start(true));
        // Wait for results
        poll_fn(|ctx| {
            if T::regs().sr().read().addrerrf() {
                // Clear address error flag
                T::regs().clrfr().write(|w| w.set_addrerrfc(true));
            } else if T::regs().sr().read().ramerrf() {
                // Clear RAM error flag
                T::regs().clrfr().write(|w| w.set_ramerrfc(true));
            }

            if T::regs().sr().read().procendf() {
                // Clear process end flag
                T::regs().clrfr().write(|w| w.set_procendfc(true));
                return Poll::Ready(());
            }

            PKA_WAKER.register(ctx.waker());
            // Enable interrupts for process end
            T::regs().cr().modify(|w| w.set_procendie(true));

            if T::regs().sr().read().procendf() {
                // Clear process end flag
                T::regs().clrfr().write(|w| w.set_procendfc(true));
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    fn set_mode(&mut self, mode: PkaOperationMode) {
        T::regs().cr().modify(|w| w.set_mode(mode as _));
    }

    fn ram_write_param(&mut self, offset: usize, data: &[u8], padding: usize) -> PkaResult<()> {
        if (data.len() + padding) % 4 != 0 {
            return Err(PkaError::InvalidInputParameter);
        }

        data.iter()
            .chain(iter::repeat(&0x00).take(padding))
            .array_chunks::<4>()
            .map(|b| u32::from_le_bytes([*b[0], *b[1], *b[2], *b[3]]))
            .enumerate()
            .for_each(|(index, value)| self.ram_write_u32(offset + index, value));

        self.raw_write_param_end(offset + (data.len() + padding) / 4);

        Ok(())
    }

    fn ram_write_u32(&mut self, offset: usize, data: u32) {
        T::regs().ram(offset).write_value(data);
    }

    fn raw_write_param_end(&mut self, offset: usize) {
        self.ram_write_u32(offset, 0x00)
    }

    fn ram_read_u32(&mut self, offset: usize) -> u32 {
        T::regs().ram(offset).read()
    }

    fn ram_read(&mut self, offset: usize, data: &mut [u8]) {
        data.chunks_mut(4).enumerate().for_each(|(index, dest)| {
            let value = self.ram_read_u32(offset + index);
            dest.copy_from_slice(&value.to_le_bytes()[..dest.len()]);
        });
    }

    fn set_aritmetic_params(&mut self, p1: &[u8], p2: &[u8], p3: Option<&[u8]>) -> PkaResult<()> {
        let mut oprerand_size = p1.len().max(p2.len());

        if let Some(p3_len) = p3.map(|p| p.len()) {
            oprerand_size = oprerand_size.max(p3_len)
        }

        if oprerand_size > INPUT_OPERAND_MAX_SIZE {
            return Err(PkaError::InvalidInputParameter);
        }

        // Length in bytes
        self.ram_write_u32(OPERAND_LENGTH_ADDR, (oprerand_size as u32) * u8::BITS);

        let padding = 4 - oprerand_size % 4;
        self.ram_write_param(ARITMETIC_OPERAND1_ADDR, p1, oprerand_size - p1.len() + padding)?;
        self.ram_write_param(ARITMETIC_OPERAND2_ADDR, p2, oprerand_size - p2.len() + padding)?;

        if let Some(p3) = p3 {
            self.ram_write_param(ARITMETIC_OPERAND3_ADDR, p3, oprerand_size - p3.len() + padding)?;
        }

        Ok(())
    }

    fn get_result(&mut self, mode: PkaOperationMode, result: &mut [u8]) -> PkaResult<usize> {
        let (offset, length) = match mode {
            // NOTE: Some of the branches should be never reached since not all PKA operations are
            // implemented.
            PkaOperationMode::MontgomeryParameterComputationOnly => {
                let bits = self.ram_read_u32(OPERAND_LENGTH_ADDR);
                let length = (bits as usize) / 8;
                (MONTGOMERY_PARAM_ADDR, length + 1)
            }
            PkaOperationMode::ArithmeticComparison => (ARITMETIC_RESULT_ADDR, 4),
            PkaOperationMode::MontgomeryMultiplication
            | PkaOperationMode::ModularSubtraction
            | PkaOperationMode::ModularReduction
            | PkaOperationMode::ModularAddition
            | PkaOperationMode::ModularInversion
            | PkaOperationMode::ArithmeticSubtraction => {
                let length = (self.ram_read_u32(OPERAND_LENGTH_ADDR) / u8::BITS) as usize;
                (ARITMETIC_RESULT_ADDR, length)
            }
            PkaOperationMode::ArithmeticMultiplication => {
                let length = (self.ram_read_u32(OPERAND_LENGTH_ADDR) / u8::BITS * 2) as usize;
                (ARITMETIC_RESULT_ADDR, length)
            }
            PkaOperationMode::ArithmeticAddition => {
                let mut size_u32 = (self.ram_read_u32(OPERAND_LENGTH_ADDR) / u32::BITS) as usize;
                // Manage the overflow of the addition
                if self.ram_read_u32(size_u32 + 500) != 0 {
                    size_u32 += 1;
                }
                (ARITMETIC_RESULT_ADDR, size_u32 * 4)
            }
            PkaOperationMode::ModularExponentiation | PkaOperationMode::ModularExponentiationFastMode => {
                let length = ((self.ram_read_u32(OPERAND_LENGTH_ADDR) + 7) / u8::BITS) as usize;
                (EXPONENT_RESULT_ADDR, length)
            }

            PkaOperationMode::MontgomeryParameterComputationThenEccScalarMultiplication
            | PkaOperationMode::EccScalarMultiplicationOnly
            | PkaOperationMode::EcdsaSign
            | PkaOperationMode::EcdsaVerification
            | PkaOperationMode::PointOnEllipticCurveFpCheck
            | PkaOperationMode::RsaCrtExponentiation => return Err(PkaError::NotSupported),
        };

        if length > result.len() {
            return Err(PkaError::ResultBufferTooShort);
        }

        self.ram_read(offset, &mut result[..length]);

        Ok(length)
    }

    /// Calculate the Montgomery parameter
    /// Input length requirement: 0 < value < 3136bits
    /// Input have to be odd integer
    /// Returns result length if success
    pub async fn montgomery_param(&mut self, input: &[u8], result: &mut [u8]) -> PkaResult<usize> {
        if input.len() > INPUT_OPERAND_MAX_SIZE {
            return Err(PkaError::InvalidInputParameter);
        }

        let bytes_to_skip = input.iter().take_while(|&&x| x == 0x00).count();
        let input_bit_size = operand_optimal_size_bits(&input[bytes_to_skip..]);
        if input_bit_size >= 3136 {
            return Err(PkaError::InvalidInputParameter);
        }

        self.ram_write_u32(OPERAND_LENGTH_ADDR, input_bit_size);
        self.ram_write_param(ARITMETIC_OPERAND3_ADDR, input, 0)?;

        let mode = PkaOperationMode::MontgomeryParameterComputationOnly;
        self.set_mode(mode);
        self.start().await;

        self.get_result(mode, result)
    }

    /// Multiply two numbers using Montgomery multiplication
    /// Operands A and B can not be larger than modulus
    /// Result is in Montgomery domain or in natural domain, depending upon the inputs nature.
    /// Returns result length if success
    pub async fn mod_multiply(&mut self, a: &[u8], b: &[u8], modulus: &[u8], result: &mut [u8]) -> PkaResult<usize> {
        self.set_aritmetic_params(a, b, Some(modulus))?;

        let mode = PkaOperationMode::MontgomeryMultiplication;
        self.set_mode(mode);
        self.start().await;

        self.get_result(mode, result)
    }

    /// Calculate the modular inversion of a number
    /// Operand A must fit into: 0 <= A < modulus
    /// Modulus must be odd integer < 2^3136
    /// Returns result length if success
    pub async fn mod_inverse(&mut self, operand: &[u8], modulus: &[u8], result: &mut [u8]) -> PkaResult<usize> {
        if modulus.len() > INPUT_OPERAND_MAX_SIZE {
            return Err(PkaError::InvalidInputParameter);
        }

        self.ram_write_u32(OPERAND_LENGTH_ADDR, operand_size_bits(modulus));

        let padding = operand_padding(modulus) + modulus.len() - operand.len();
        self.ram_write_param(ARITMETIC_OPERAND1_ADDR, operand, padding)?;

        self.ram_write_param(ARITMETIC_OPERAND2_ADDR, modulus, operand_padding(modulus))?;

        let mode = PkaOperationMode::ModularInversion;
        self.set_mode(mode);
        self.start().await;

        self.get_result(mode, result)
    }

    /// Modular Reduction: A mod n
    /// Modulus length in bits must be 8 < mod_len_bits < 3136, modulus have to be odd integer
    /// Operand A must fit into: 0 <= A < 2n < 2^3136
    /// Returns result length if success
    pub async fn mod_reduction(&mut self, operand: &[u8], modulus: &[u8], result: &mut [u8]) -> PkaResult<usize> {
        if operand.len() > INPUT_OPERAND_MAX_SIZE || modulus.len() > INPUT_OPERAND_MAX_SIZE || modulus.len() < 1 {
            return Err(PkaError::InvalidInputParameter);
        }

        self.ram_write_u32(MODULUS_LENGTH_ADDR, operand_size_bits(modulus));
        self.ram_write_u32(OPERAND_LENGTH_ADDR, operand_size_bits(operand));

        self.ram_write_param(ARITMETIC_OPERAND1_ADDR, operand, operand_padding(operand))?;
        // TODO: Why I need add larger padding to modulus to make it working?
        self.ram_write_param(ARITMETIC_OPERAND2_ADDR, modulus, operand_padding(modulus) + 4)?;

        let mode = PkaOperationMode::ModularReduction;
        self.set_mode(mode);
        self.start().await;

        self.get_result(mode, result)
    }

    /// Calculate the modular exponentiation A^e mod n
    /// Modes:
    ///    r2_mod_n is None => Normal Mode
    ///    r2_mod_n is Some => Fast Mode
    /// Operand A have to be 0 <= A < n
    /// Exponent e have to be 0 <= e < n
    /// Modulus n have to be odd integer and n < 2^3136
    pub async fn mod_exponent(
        &mut self,
        operand: &[u8],
        exponent: &[u8],
        modulus: &[u8],
        r2_mod_n: Option<&[u8]>,
        result: &mut [u8],
    ) -> PkaResult<usize> {
        if operand.len() > INPUT_OPERAND_MAX_SIZE
            || exponent.len() > INPUT_OPERAND_MAX_SIZE
            || modulus.len() > INPUT_OPERAND_MAX_SIZE
        {
            return Err(PkaError::InvalidInputParameter);
        }

        self.ram_write_u32(EXPONENT_LENGTH_ADDR, operand_size_bits(exponent));
        self.ram_write_u32(OPERAND_LENGTH_ADDR, operand_size_bits(operand));

        self.ram_write_param(ARITMETIC_OPERAND2_ADDR, operand, operand_padding(operand))?;
        self.ram_write_param(EXPONENT_IN_ADDR, exponent, operand_padding(exponent))?;

        let modulus_padding = operand.len() + operand_padding(operand) - modulus.len();
        self.ram_write_param(ARITMETIC_OPERAND3_ADDR, modulus, modulus_padding)?;

        let mode = match r2_mod_n {
            Some(r2_mod_n) => {
                self.ram_write_param(MONTGOMERY_PARAM_ADDR, r2_mod_n, operand_padding(r2_mod_n))?;

                PkaOperationMode::ModularExponentiationFastMode
            }
            None => PkaOperationMode::ModularExponentiation,
        };

        self.set_mode(mode);
        self.start().await;

        self.get_result(mode, result)
    }

    /// Calculate A * B
    /// Returns result length if success
    /// Result buffer have to be at least 2 * (operand size)
    pub async fn arithmetic_multiply(&mut self, a: &[u8], b: &[u8], result: &mut [u8]) -> PkaResult<usize> {
        self.set_aritmetic_params(a, b, None)?;

        let mode = PkaOperationMode::ArithmeticMultiplication;
        self.set_mode(mode);
        self.start().await;

        self.get_result(mode, result)
    }
}

fn operand_size_bits(operand: &[u8]) -> u32 {
    (operand.len() as u32) * u8::BITS
}

fn operand_optimal_size_bits(operand: &[u8]) -> u32 {
    if operand.len() == 0 {
        return 0;
    }

    let position = u8::BITS - operand[0].leading_zeros();
    return (((operand.len() as u32) - 1) * u8::BITS) + position;
}

/// Returns operand padding which is required to fill [u32] PKA ram
fn operand_padding(operand: &[u8]) -> usize {
    (4 - operand.len() % 4) % 4
}

/// PKA instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this PKA instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, pka, PKA, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::pka::Pka {
                crate::pac::$inst
            }
        }
    };
);
