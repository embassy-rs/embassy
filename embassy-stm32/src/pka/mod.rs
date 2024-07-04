//! PKA Hardware accelerator
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;
use heapless::Vec;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc};

static PKA_WAKER: AtomicWaker = AtomicWaker::new();

// List of RAM addresses for data. Since embassy implementation of RAM access already moves base address to
// 0x0400 and multiplies offset by 4 for 32-bit access, we have to take address from reference manual, subtract 0x0400
// and divide it by 4 to get the correct offset.
const OPERAND_LENGTH_ADDR: usize = 0x0001; // 0x404
const MODULUS_ADDR: usize = 0x0257; // 0xD5C
const MONT_PARAM_R2_MOD_N: usize = 0x0065; // 0x594
const OPERAND_A_ADDR: usize = 0x012D; // 0x8B4
const OPERAND_B_ADDR: usize = 0x0191; // 0xA44
const ARITHMETIC_RESULT_ADDR: usize = 0x01F4; // 0xBD0
const EXPONENT_LENGTH_ADDR: usize = 0x0000; // 0x400
const EXPONENT_ADDR: usize = ARITHMETIC_RESULT_ADDR; // 0xBD0
const EXPONENTIATION_RESULT_ADDR: usize = 0x00C9; // 0x724

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
        info!("PKA interrupt");
        let procendf = T::regs().sr().read().procendf();
        let addr_err = T::regs().sr().read().addrerrf();
        let ram_err = T::regs().sr().read().ramerrf();

        if procendf {
            info!("PKA interrupt: procendf");
            T::regs().cr().modify(|w| w.set_procendie(false));
        }

        if addr_err {
            info!("PKA interrupt: addr_err");
            T::regs().cr().modify(|w| w.set_addrerrie(false));
        }

        if ram_err {
            info!("PKA interrupt: ram_err");
            T::regs().cr().modify(|w| w.set_ramerrie(false));
        }

        PKA_WAKER.wake();
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// RSA structure for PKA accelerator
pub struct Rsa<'c, 'd, T: Instance, const RSA_KEY_SIZE: usize> {
    /// Modulus
    pub modulus: [u32; RSA_KEY_SIZE],
    /// R^2 mod N
    r2_mod_n: [u32; RSA_KEY_SIZE],
    /// public exponent
    pub public_exponent: [u32; RSA_KEY_SIZE],
    /// private exponent
    /// FIXME: In some edge cases the private exponent can be larger than the modulus
    pub private_exponent: [u32; RSA_KEY_SIZE],
    /// PKA accelerator
    pka: &'c mut Pka<'d, T, RSA_KEY_SIZE>,
}

impl<'c, 'd, T: Instance, const RSA_KEY_SIZE: usize> Rsa<'c, 'd, T, RSA_KEY_SIZE> {
    /// Create a new RSA structure
    pub fn new(pka: &'c mut Pka<'d, T, RSA_KEY_SIZE>) -> Self {
        pka.enable();
        Self {
            modulus: [0; RSA_KEY_SIZE],
            r2_mod_n: [0; RSA_KEY_SIZE],
            public_exponent: [0; RSA_KEY_SIZE],
            private_exponent: [0; RSA_KEY_SIZE],
            pka,
        }
    }

    /// Generate RSA key pair
    pub async fn generate_key_pair(&mut self, prime_q: &mut [u32], prime_p: &mut [u32]) {
        // Generate public and private keys
        if prime_p.len() > RSA_KEY_SIZE / 2 || prime_q.len() > RSA_KEY_SIZE / 2 {
            panic!(
                "Prime numbers are too large. They should be less or equal to {:?} / 2, but is {:?} and {:?} respectively",
                RSA_KEY_SIZE, prime_p.len(), prime_q.len()
            );
        }

        info!("Prime P: {:x}", prime_p);
        info!("Prime Q: {:x}", prime_q);

        // Generate modulus
        // FIXME: Currently modulus is hardcoded to 32 bits
        self.modulus = self.pka.arithmetic_multiply(prime_p, prime_q).await.unwrap();

        // Generate R^2 mod N
        self.r2_mod_n = self.pka.montgomery_param(&self.modulus).await.unwrap();

        // Compute Eueler's totient function
        // Slice of primes. Same length, but last element is decreased by 1
        let prime_p_minus_1 = prime_p;
        prime_p_minus_1[0] -= 1;

        let prime_q_minus_1 = prime_q;
        prime_q_minus_1[0] -= 1;

        let phi = self
            .pka
            .arithmetic_multiply(&prime_p_minus_1, &prime_q_minus_1)
            .await
            .unwrap();

        // Choose public exponent. It is usually 65537
        self.public_exponent[0] = 65537;

        // Generate private key
        info!("Generating private exponent");
        self.private_exponent = self.pka.mod_inverse(&self.public_exponent, &phi).await.unwrap();
    }
    /// Encrypt data
    pub async fn encrypt(&mut self, data: &[u32]) -> Result<[u32; RSA_KEY_SIZE], ()> {
        // Message to encrypt should be less than modulus
        if data.len() > RSA_KEY_SIZE {
            panic!(
                "Data is too large. It should be less or equal to {:?}, but is {:?}",
                RSA_KEY_SIZE,
                data.len()
            );
        }

        let result = self
            .pka
            .mod_exp(data, &self.public_exponent, &self.modulus, &self.r2_mod_n)
            .await;
        result
    }

    pub async fn test(&mut self) {
        let e = 3;
        let n = 33;
        let d = 7;
        let message = 4;

        let encrypted = self.pka.mod_exp(&[message], &[e], &[n], &[0]).await;
        info!("Encrypted: {:x}", encrypted);

        let decrypted = self.pka.mod_exp(&encrypted.unwrap(), &[d], &[n], &[0]).await;
        info!("Decrypted: {:x}", decrypted);
    }

    /// Decrypt data
    pub async fn decrypt(&mut self, data: &[u32]) -> Result<[u32; RSA_KEY_SIZE], ()> {
        // Message to decrypt must be exactly the size of modulus
        if data.len() != RSA_KEY_SIZE {
            panic!(
                "Data length invalid. It must be {:?}, but is {:?}",
                RSA_KEY_SIZE,
                data.len()
            );
        }

        let result = self
            .pka
            .mod_exp(data, &self.private_exponent, &self.modulus, &self.r2_mod_n)
            .await;
        result
    }
}

/// PKA accelerator
pub struct Pka<'d, T, const RSA_KEY_SIZE: usize> {
    _peripheral: PeripheralRef<'d, T>,
}

impl<'d, T: Instance, const RSA_KEY_SIZE: usize> Pka<'d, T, RSA_KEY_SIZE> {
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
                info!("Address error");
                T::regs().clrfr().write(|w| w.set_addrerrfc(true));
            } else if T::regs().sr().read().ramerrf() {
                info!("RAM error");
                T::regs().clrfr().write(|w| w.set_ramerrfc(true));
            }

            if T::regs().sr().read().procendf() {
                info!("Process end");
                T::regs().clrfr().write(|w| w.set_procendfc(true));
                return Poll::Ready(());
            }

            PKA_WAKER.register(ctx.waker());
            T::regs().cr().modify(|w| w.set_procendie(true));

            if T::regs().sr().read().procendf() {
                info!("Process end");
                T::regs().clrfr().write(|w| w.set_procendfc(true));
                info!("Process end cleared");
                Poll::Ready(())
            } else {
                info!("Waiting for PKA");
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
    fn write_to_ram(&mut self, offset: usize, data: &[u32]) {
        for (i, byte) in data.iter().enumerate() {
            T::regs().ram(offset + i).write(|w| {
                *w = *byte;
                info!("write_to_ram: 0x{:x} at offset 0x{:x}", *byte, offset + i);
                *w
            });
        }
        // Write 0 to next word
        T::regs().ram(offset + data.len()).write(|w| {
            *w = 0;
            *w
        });
    }

    /// Write to PKA RAM with offset. Don't set the last word to 0
    fn write_to_ram_u32(&mut self, offset: usize, data: &u32) {
        T::regs().ram(offset).write(|w| {
            *w = *data;
            info!("write_to_ram: 0x{:x} at offset 0x{:x}", *data, offset);
            *w
        });
    }

    /// Read from PKA RAM with offset
    fn read_from_ram(&mut self, offset: usize, data: &mut [u32]) {
        // Read from PKA RAM with offset. RAM is little endian, so we have to convert it
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = T::regs().ram(offset + i).read();
            info!("read_from_ram: 0x{:x} at offset 0x{:x}", *byte, offset + i);
        }
    }

    /// Calculate the Montgomery parameter
    pub async fn montgomery_param(&mut self, mod_value: &[u32]) -> Result<[u32; RSA_KEY_SIZE], ()> {
        // Get the size of a value in modulus in bits
        // FIXME: Currently modulus size is hardcoded to 32 bits.
        // It has to be calculated from the modulus value, which is an array of u32

        // Modulus can only be odd number
        if mod_value[0] % 2 == 0 {
            return Err(());
        }

        let modulus_size = [32 * (mod_value.len() as u32)];

        self.write_to_ram(OPERAND_LENGTH_ADDR, &modulus_size);
        self.write_to_ram(MODULUS_ADDR, mod_value);

        let mode = PkaOperationMode::MontgomeryParameterComputationOnly as u8;
        self.set_mode(mode);

        self.start().await;

        let mut result: [u32; RSA_KEY_SIZE] = [0; RSA_KEY_SIZE];
        self.read_from_ram(MONT_PARAM_R2_MOD_N, &mut result);

        Ok(result)
    }

    /// Multiply two numbers using Montgomery multiplication
    pub async fn modular_multiply(&mut self, a: &[u32], b: &[u32], modulus: &[u32]) -> Result<[u32; RSA_KEY_SIZE], ()> {
        //FIXME: Currently modulus size is hardcoded to 32 bits. Same for the result

        // Calculate the maximum size of operands based on leading zeros
        let operand_size: [u32; 1] = [RSA_KEY_SIZE as u32];

        self.write_to_ram(OPERAND_LENGTH_ADDR, &operand_size);
        self.write_to_ram(MODULUS_ADDR, modulus);

        self.write_to_ram(OPERAND_A_ADDR, &a);
        self.write_to_ram(OPERAND_B_ADDR, &b);

        let mode = PkaOperationMode::MontgomeryMultiplication as u8;
        self.set_mode(mode);

        self.start().await;

        let mut result: [u32; RSA_KEY_SIZE] = [0; RSA_KEY_SIZE];
        self.read_from_ram(MONT_PARAM_R2_MOD_N, &mut result);

        Ok(result)
    }

    /// Calculate the modular inversion of a number
    pub async fn mod_inverse(&mut self, a: &[u32], modulus: &[u32]) -> Result<[u32; RSA_KEY_SIZE], ()> {
        // Calculate the operand size.
        let operand_size: [u32; 1] = [32 * 4];

        self.write_to_ram(OPERAND_LENGTH_ADDR, &operand_size);
        self.write_to_ram(OPERAND_A_ADDR, &a);
        // This is not an error. Once in a while modulus is written to different address
        self.write_to_ram(OPERAND_B_ADDR, modulus);

        let mode = PkaOperationMode::ModularInversion as u8;
        self.set_mode(mode);

        self.start().await;

        let mut result: [u32; RSA_KEY_SIZE] = [0; RSA_KEY_SIZE];
        self.read_from_ram(ARITHMETIC_RESULT_ADDR, &mut result);

        Ok(result)
    }

    /// Calculate the modular exponentiation
    pub async fn mod_exp(
        &mut self,
        operand: &[u32],
        exponent: &[u32],
        modulus: &[u32],
        r2_mod_n: &[u32],
    ) -> Result<[u32; RSA_KEY_SIZE], ()> {
        let mut result: [u32; RSA_KEY_SIZE] = [0; RSA_KEY_SIZE];

        let exponent_size: u32 = 2;

        // Calculate the operand size
        let operand_size: u32 = 6;

        info!("Operand {:?}", operand);

        self.write_to_ram_u32(EXPONENT_LENGTH_ADDR, &exponent_size);
        self.write_to_ram_u32(OPERAND_LENGTH_ADDR, &operand_size);
        self.write_to_ram(OPERAND_A_ADDR, operand);
        self.write_to_ram(EXPONENT_ADDR, exponent);
        self.write_to_ram(MODULUS_ADDR, modulus);
        // self.write_to_ram(MONT_PARAM_R2_MOD_N, r2_mod_n);

        let mode = PkaOperationMode::MontgomeryParameterComputationThenModularExponentiation as u8;
        self.set_mode(mode);

        self.start().await;

        self.read_from_ram(EXPONENTIATION_RESULT_ADDR, &mut result);

        // 029787fb c05876d5a e79aad9 1e8af815
        Ok(result)
    }

    /// Calculate A * B
    pub async fn arithmetic_multiply(&mut self, a: &[u32], b: &[u32]) -> Result<[u32; RSA_KEY_SIZE], ()> {
        // Calculate the operand size
        // FIXME: Hardcoded, remove later
        let mut operand_size: [u32; 1] = [0];

        for i in 0..a.len() {
            if a[i] != 0 {
                continue;
            }
            info!("Current iteration: {:?}", i);
            operand_size[0] = 32 * (i as u32);
            break;
        }

        if operand_size[0] == 0 {
            operand_size[0] = 32 * (a.len() as u32);
        }

        info!("Operand size: {:?}", operand_size);

        self.write_to_ram(OPERAND_LENGTH_ADDR, &operand_size);
        self.write_to_ram(OPERAND_A_ADDR, &a);
        self.write_to_ram(OPERAND_B_ADDR, &b);

        let mode = PkaOperationMode::ArithmeticMultiplication as u8;
        self.set_mode(mode);

        self.start().await;

        let mut result: [u32; RSA_KEY_SIZE] = [0; RSA_KEY_SIZE];
        self.read_from_ram(ARITHMETIC_RESULT_ADDR, &mut result);

        // Fill "a" variable with the zeroes
        Ok(result)
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
