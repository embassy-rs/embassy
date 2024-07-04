//! PKA Hardware accelerator
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;
use core::{fmt, usize};

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc};

static PKA_WAKER: AtomicWaker = AtomicWaker::new();

// List of RAM addresses for data. Since embassy implementation of RAM access already moves base address to
// 0x0400 and multiplies offset by 4 for 32-bit access, we have to take address from reference manual, subtract 0x0400
// and divide it by 4 to get the correct offset.
enum MontParamComputation {
    ModulusLength,
    Modulus,
    Result,
}

impl MontParamComputation {
    const MODULUS_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0D5C - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0594 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            MontParamComputation::ModulusLength => Self::MODULUS_LENGTH_ADDR,
            MontParamComputation::Modulus => Self::MODULUS_ADDR,
            MontParamComputation::Result => Self::RESULT_ADDR,
        }
    }
}

enum ModularAddition {
    OperandLength,
    OperandA,
    OperandB,
    Modulus,
    Result,
}

impl ModularAddition {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0D5C - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ModularAddition::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ModularAddition::OperandA => Self::OPERAND_A_ADDR,
            ModularAddition::OperandB => Self::OPERAND_B_ADDR,
            ModularAddition::Modulus => Self::MODULUS_ADDR,
            ModularAddition::Result => Self::RESULT_ADDR,
        }
    }
}

enum ModularSubtraction {
    OperandLength,
    OperandA,
    OperandB,
    Modulus,
    Result,
}

impl ModularSubtraction {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0D5C - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ModularSubtraction::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ModularSubtraction::OperandA => Self::OPERAND_A_ADDR,
            ModularSubtraction::OperandB => Self::OPERAND_B_ADDR,
            ModularSubtraction::Modulus => Self::MODULUS_ADDR,
            ModularSubtraction::Result => Self::RESULT_ADDR,
        }
    }
}

enum ModularMultiplication {
    OperandLength,
    OperandA,
    OperandB,
    Modulus,
    Result,
}

impl ModularMultiplication {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0D5C - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ModularMultiplication::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ModularMultiplication::OperandA => Self::OPERAND_A_ADDR,
            ModularMultiplication::OperandB => Self::OPERAND_B_ADDR,
            ModularMultiplication::Modulus => Self::MODULUS_ADDR,
            ModularMultiplication::Result => Self::RESULT_ADDR,
        }
    }
}

enum ModularExponentiation {
    ExponentLength,
    OperandLength,
    OperandA,
    Exponent,
    Modulus,
    R2ModN,
    Result,
}

impl ModularExponentiation {
    const EXPONENT_LENGTH_ADDR: usize = (0x0400 - 0x0400) / 4;
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const EXPONENT_ADDR: usize = (0x0BD0 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0D5C - 0x0400) / 4;
    const R2_MOD_N_ADDR: usize = (0x0594 - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0724 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ModularExponentiation::ExponentLength => Self::EXPONENT_LENGTH_ADDR,
            ModularExponentiation::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ModularExponentiation::OperandA => Self::OPERAND_A_ADDR,
            ModularExponentiation::Exponent => Self::EXPONENT_ADDR,
            ModularExponentiation::Modulus => Self::MODULUS_ADDR,
            ModularExponentiation::R2ModN => Self::R2_MOD_N_ADDR,
            ModularExponentiation::Result => Self::RESULT_ADDR,
        }
    }
}

enum ModularInversion {
    OperandLength,
    OperandA,
    Modulus,
    Result,
}

impl ModularInversion {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ModularInversion::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ModularInversion::OperandA => Self::OPERAND_A_ADDR,
            ModularInversion::Modulus => Self::MODULUS_ADDR,
            ModularInversion::Result => Self::RESULT_ADDR,
        }
    }
}

enum MontgomeryMultiplication {
    OperandLength,
    OperandA,
    OperandB,
    Modulus,
    Result,
}

impl MontgomeryMultiplication {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0D5C - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            MontgomeryMultiplication::OperandLength => Self::OPERAND_LENGTH_ADDR,
            MontgomeryMultiplication::OperandA => Self::OPERAND_A_ADDR,
            MontgomeryMultiplication::OperandB => Self::OPERAND_B_ADDR,
            MontgomeryMultiplication::Modulus => Self::MODULUS_ADDR,
            MontgomeryMultiplication::Result => Self::RESULT_ADDR,
        }
    }
}

enum ModularReduction {
    OperandLength,
    ModulusLength,
    OperandA,
    Modulus,
    Result,
}

impl ModularReduction {
    const OPERAND_LENGTH_ADDR: usize = (0x0400 - 0x0400) / 4;
    const MODULUS_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const MODULUS_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ModularReduction::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ModularReduction::ModulusLength => Self::MODULUS_LENGTH_ADDR,
            ModularReduction::OperandA => Self::OPERAND_A_ADDR,
            ModularReduction::Modulus => Self::MODULUS_ADDR,
            ModularReduction::Result => Self::RESULT_ADDR,
        }
    }
}

enum ArithmeticAddition {
    OperandLength,
    OperandA,
    OperandB,
    Result,
}

impl ArithmeticAddition {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ArithmeticAddition::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ArithmeticAddition::OperandA => Self::OPERAND_A_ADDR,
            ArithmeticAddition::OperandB => Self::OPERAND_B_ADDR,
            ArithmeticAddition::Result => Self::RESULT_ADDR,
        }
    }
}

enum ArithmeticSubtraction {
    OperandLength,
    OperandA,
    OperandB,
    Result,
}

impl ArithmeticSubtraction {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ArithmeticSubtraction::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ArithmeticSubtraction::OperandA => Self::OPERAND_A_ADDR,
            ArithmeticSubtraction::OperandB => Self::OPERAND_B_ADDR,
            ArithmeticSubtraction::Result => Self::RESULT_ADDR,
        }
    }
}

enum ArithmeticMultiplication {
    OperandLength,
    OperandA,
    OperandB,
    Result,
}

impl ArithmeticMultiplication {
    const OPERAND_LENGTH_ADDR: usize = (0x0404 - 0x0400) / 4;
    const OPERAND_A_ADDR: usize = (0x08B4 - 0x0400) / 4;
    const OPERAND_B_ADDR: usize = (0x0A44 - 0x0400) / 4;
    const RESULT_ADDR: usize = (0x0BD0 - 0x0400) / 4;

    fn value(&self) -> usize {
        match self {
            ArithmeticMultiplication::OperandLength => Self::OPERAND_LENGTH_ADDR,
            ArithmeticMultiplication::OperandA => Self::OPERAND_A_ADDR,
            ArithmeticMultiplication::OperandB => Self::OPERAND_B_ADDR,
            ArithmeticMultiplication::Result => Self::RESULT_ADDR,
        }
    }
}

enum ArithmeticComparison {
    OperandLength,
    OperandA,
    OperandB,
    Result,
}

impl ArithmeticComparison {
    fn value(&self) -> usize {
        match self {
            ArithmeticComparison::OperandLength => 0x0404,
            ArithmeticComparison::OperandA => 0x08B4,
            ArithmeticComparison::OperandB => 0x0A44,
            ArithmeticComparison::Result => 0x0BD0,
        }
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// PKA Operation Mode
pub enum PkaOperationMode {
    /// Montgomery Parameter Computation then Modular Exponentiation
    MontgomeryParameterComputationThenModularExponentiation = 0b000000,
    /// Montgomery Parameter Computation Only
    MontgomeryParameterComputationOnly = 0b000001,
    /// Modular Exponentiation Only. (Montgomery parameter must be loaded first)
    ModularExponentiationOnly = 0b000010,
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

#[derive(Debug)]
/// Error codes for PKA
pub enum RsaErrorCodes {
    /// Primes too large
    PrimesTooLarge,
    /// Message too long
    MessageLengthInvalid,
}

#[allow(dead_code)]
/// Error type for PKA
pub struct RsaError {
    /// Error code
    err_code: RsaErrorCodes,
    /// Error message
    message: &'static str,
}

impl fmt::Debug for RsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error code: {:?}, Message: {}", self.err_code, self.message)
    }
}

impl fmt::Display for RsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error code: {:?}, Message: {}", self.err_code, self.message)
    }
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// RSA structure for PKA accelerator
pub struct Rsa<const RSA_KEY_SIZE: usize> {
    /// Modulus
    pub modulus: [u8; RSA_KEY_SIZE],
    /// R^2 mod N
    r2_mod_n: [u8; RSA_KEY_SIZE],
    /// public exponent
    pub public_exponent: [u8; RSA_KEY_SIZE],
    /// private exponent
    /// FIXME: In some edge cases the private exponent can be larger than the modulus
    pub private_exponent: [u8; RSA_KEY_SIZE],
}

impl<const RSA_KEY_SIZE: usize> Rsa<RSA_KEY_SIZE> {
    /// Create a new RSA structure
    pub fn new() -> Self {
        Self {
            modulus: [0; RSA_KEY_SIZE],
            r2_mod_n: [0; RSA_KEY_SIZE],
            public_exponent: [0; RSA_KEY_SIZE],
            private_exponent: [0; RSA_KEY_SIZE],
        }
    }

    /// Create RSA structure from private key components
    pub async fn from_private_key<'c, 'd, T: Instance>(
        pka: &'c mut Pka<'d, T>,
        private_exponent: &[u8],
        modulus: &[u8],
    ) -> Self {
        if private_exponent.len() > RSA_KEY_SIZE || modulus.len() > RSA_KEY_SIZE {
            panic!("Private exponent or modulus is too large");
        }

        #[cfg(feature = "defmt")]
        defmt::debug!("Generating RSA key pair from private key and modulus");

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Private exponent size: {}, modulus size: {}",
            private_exponent.len(),
            modulus.len()
        );

        let mut public_exponent: [u8; RSA_KEY_SIZE] = [0; RSA_KEY_SIZE];

        public_exponent[0] = 0x01;
        public_exponent[1] = 0x00;
        public_exponent[2] = 0x01;

        pka.enable();

        let mut r2_mod_n = [0; RSA_KEY_SIZE];
        pka.montgomery_param(&modulus, &mut r2_mod_n).await;

        // Copy data from modulus slice to modulus array
        let mut modulus_array = [0; RSA_KEY_SIZE];
        modulus_array.copy_from_slice(modulus);

        // Same for private exponent
        let mut private_exponent_array = [0; RSA_KEY_SIZE];
        private_exponent_array.copy_from_slice(private_exponent);

        Self {
            modulus: modulus_array,
            r2_mod_n,
            public_exponent,
            private_exponent: private_exponent_array,
        }
    }

    /// Reorder data for the new word order
    fn word_reorder<const N: usize>(input: &[u8; N], result: &mut [u8; N]) {
        // Ensure the input length is a multiple of 4 by padding with 0s if necessary

        input
            .chunks(4)
            .rev()
            .enumerate()
            .for_each(|(i, c)| result[(i * 4)..(i * 4 + 4)].copy_from_slice(c));
    }

    /// Generate RSA key pair
    pub async fn generate_key_pair<'c, 'd, T: Instance, const N: usize>(
        &mut self,
        pka: &'c mut Pka<'d, T>,
        prime_1: [u8; N],
        prime_2: [u8; N],
    ) -> Result<(), RsaError> {
        // Generate public and private keys
        pka.enable();
        if prime_2.len() > RSA_KEY_SIZE / 2 || prime_1.len() > RSA_KEY_SIZE / 2 {
            return Err(RsaError {
                err_code: RsaErrorCodes::PrimesTooLarge,
                message: "Prime numbers are too large",
            });
        }

        // Generate modulus
        pka.arithmetic_multiply(&prime_1, &prime_2, &mut self.modulus).await;

        // Generate R^2 mod N
        pka.montgomery_param(&self.modulus, &mut self.r2_mod_n).await;

        // Compute Eueler's totient function
        // Slice of primes. Same length, but last element is decreased by 1
        let mut prime_2_minus_1 = prime_1;
        prime_2_minus_1[0] -= 1;

        let mut prime_1_minus_1 = prime_2;
        prime_1_minus_1[0] -= 1;

        let mut phi = [0; RSA_KEY_SIZE];
        pka.arithmetic_multiply(&prime_2_minus_1, &prime_1_minus_1, &mut phi)
            .await;

        // Choose public exponent. It is usually 65537
        self.public_exponent[0] = 0x01;
        self.public_exponent[1] = 0x00;
        self.public_exponent[2] = 0x01;

        // Generate private key
        pka.mod_inverse(&self.public_exponent, &phi, &mut self.private_exponent)
            .await;

        Ok(())
    }

    /// Encrypt data
    pub async fn encrypt<'c, 'd, T: Instance, const N: usize>(
        &mut self,
        pka: &'c mut Pka<'d, T>,
        data: &[u8; N],
        result: &mut [u8; N],
    ) -> Result<(), RsaError> {
        // Message to encrypt should be less than modulus
        if data.len() > RSA_KEY_SIZE {
            return Err(RsaError {
                err_code: RsaErrorCodes::MessageLengthInvalid,
                message: "Message is too long. It must be less than RSA key size",
            });
        }

        let mut reordered_data = [0; N];
        Self::word_reorder(data, &mut reordered_data);

        pka.mod_exp(
            &reordered_data,
            &self.public_exponent,
            &self.modulus,
            &self.r2_mod_n,
            result,
        )
        .await;

        Ok(())
    }

    /// Decrypt data
    pub async fn decrypt<'c, 'd, T: Instance, const N: usize>(
        &mut self,
        pka: &'c mut Pka<'d, T>,
        data: &[u8; N],
        result: &mut [u8; N],
    ) -> Result<(), RsaError> {
        // Message to decrypt must be exactly the size of modulus
        if data.len() != RSA_KEY_SIZE {
            return Err(RsaError {
                err_code: RsaErrorCodes::MessageLengthInvalid,
                message: "Encrypted message length is not equal to RSA key size",
            });
        }

        let mut reordered_data = [0; N];

        pka.mod_exp(
            data,
            &self.private_exponent,
            &self.modulus,
            &self.r2_mod_n,
            &mut reordered_data,
        )
        .await;
        Self::word_reorder(&reordered_data, result);

        Ok(())
    }
}

/// PKA accelerator
pub struct Pka<'d, T> {
    _peripheral: PeripheralRef<'d, T>,
}

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
    fn disable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(false));
    }

    /// Enable PKA accelerator
    fn enable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(true));
    }

    /// Start PKA operation
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

    /// Set PKA working mode
    fn set_mode(&mut self, mode: u8) {
        T::regs().cr().modify(|w| w.set_mode(mode));
    }

    /// Write to PKA RAM with offset
    fn write_to_ram(&mut self, offset: usize, data: &[u8]) {
        let len = data.len() / 4;
        let u32_data = data
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));

        for (i, byte) in u32_data.enumerate() {
            T::regs().ram(offset + i).write_value(byte);
        }

        // Write 0 to next word
        T::regs().ram(offset + len).write_value(0);
    }

    /// Write to PKA RAM with offset. Don't set the last word to 0
    fn write_to_ram_u32(&mut self, offset: usize, data: &u32) {
        T::regs().ram(offset).write_value(*data);
    }

    /// Read from PKA RAM with offset
    fn read_from_ram(&mut self, offset: usize, data: &mut [u8]) {
        // Read from PKA RAM with offset. RAM is little endian
        for i in 0..data.len() / 4 {
            let byte = T::regs().ram(offset + i).read();
            // Copy u32 to u8
            let u8_data = byte.to_le_bytes();
            data[i * 4..(i + 1) * 4].copy_from_slice(&u8_data);
        }
    }

    /// Calculate the Montgomery parameter
    pub async fn montgomery_param(&mut self, mod_value: &[u8], result: &mut [u8]) {
        // Get the size of a value in modulus in bits
        // It has to be calculated from the modulus value, which is an array of u8

        let modulus_size = 8 * (mod_value.len() as u32);

        self.write_to_ram_u32(MontParamComputation::ModulusLength.value(), &modulus_size);
        self.write_to_ram(MontParamComputation::Modulus.value(), mod_value);

        let mode = PkaOperationMode::MontgomeryParameterComputationOnly as u8;
        self.set_mode(mode);

        self.start().await;

        self.read_from_ram(MontParamComputation::Result.value(), result);
    }

    /// Multiply two numbers using Montgomery multiplication
    pub async fn modular_multiply(&mut self, a: &[u8], b: &[u8], modulus: &[u8], result: &mut [u8]) {
        // Calculate the maximum size of operands based on leading zeros
        let operand_size = 8 * (a.len() as u32);

        self.write_to_ram_u32(ModularMultiplication::OperandLength.value(), &operand_size);

        self.write_to_ram(ModularMultiplication::Modulus.value(), modulus);
        self.write_to_ram(ModularMultiplication::OperandA.value(), &a);
        self.write_to_ram(ModularMultiplication::OperandB.value(), &b);

        let mode = PkaOperationMode::MontgomeryMultiplication as u8;
        self.set_mode(mode);

        self.start().await;

        self.read_from_ram(ModularMultiplication::Result.value(), result);
    }

    /// Calculate the modular inversion of a number
    pub async fn mod_inverse(&mut self, a: &[u8], modulus: &[u8], result: &mut [u8]) {
        // Calculate the operand size.
        let operand_size = 8 * (modulus.len() as u32);

        self.write_to_ram_u32(ModularInversion::OperandLength.value(), &operand_size);

        self.write_to_ram(ModularInversion::OperandA.value(), &a);
        // This is not an error. Once in a while modulus is written to different address
        self.write_to_ram(ModularInversion::Modulus.value(), modulus);

        let mode = PkaOperationMode::ModularInversion as u8;
        self.set_mode(mode);

        self.start().await;

        self.read_from_ram(ModularInversion::Result.value(), result);
    }

    /// Calculate the modular exponentiation
    pub async fn mod_exp(
        &mut self,
        operand: &[u8],
        exponent: &[u8],
        modulus: &[u8],
        r2_mod_n: &[u8],
        result: &mut [u8],
    ) {
        let exponent_size = 8 * exponent.len() as u32 - exponent[0].leading_zeros() as u32;

        // Calculate the operand size
        let operand_size = 8 * modulus.len() as u32;

        self.write_to_ram_u32(ModularExponentiation::ExponentLength.value(), &exponent_size);
        self.write_to_ram_u32(ModularExponentiation::OperandLength.value(), &operand_size);

        self.write_to_ram(ModularExponentiation::OperandA.value(), operand);
        self.write_to_ram(ModularExponentiation::Exponent.value(), exponent);
        self.write_to_ram(ModularExponentiation::Modulus.value(), modulus);
        self.write_to_ram(ModularExponentiation::R2ModN.value(), r2_mod_n);

        let mode = PkaOperationMode::ModularExponentiationOnly as u8;
        self.set_mode(mode);

        self.start().await;

        self.read_from_ram(ModularExponentiation::Result.value(), result);
    }

    /// Calculate A * B
    pub async fn arithmetic_multiply(&mut self, a: &[u8], b: &[u8], result: &mut [u8]) {
        // Calculate the operand size

        let operand_size = 8 * (a.len() as u32);

        self.write_to_ram_u32(ArithmeticMultiplication::OperandLength.value(), &operand_size);
        self.write_to_ram(ArithmeticMultiplication::OperandA.value(), &a);
        self.write_to_ram(ArithmeticMultiplication::OperandB.value(), &b);

        let mode = PkaOperationMode::ArithmeticMultiplication as u8;
        self.set_mode(mode);

        self.start().await;

        self.read_from_ram(ArithmeticMultiplication::Result.value(), result);
    }
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
