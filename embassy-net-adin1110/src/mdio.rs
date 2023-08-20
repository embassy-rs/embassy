/// PHY Address: (0..=0x1F), 5-bits long.
#[allow(dead_code)]
type PhyAddr = u8;

/// PHY Register: (0..=0x1F), 5-bits long.
#[allow(dead_code)]
type RegC22 = u8;

/// PHY Register Clause 45.
#[allow(dead_code)]
type RegC45 = u16;

/// PHY Register Value
#[allow(dead_code)]
type RegVal = u16;

#[allow(dead_code)]
const REG13: RegC22 = 13;
#[allow(dead_code)]
const REG14: RegC22 = 14;

#[allow(dead_code)]
const PHYADDR_MASK: u8 = 0x1f;
#[allow(dead_code)]
const DEV_MASK: u8 = 0x1f;

#[allow(dead_code)]
#[repr(u16)]
enum Reg13Op {
    Addr = 0b00 << 14,
    Write = 0b01 << 14,
    PostReadIncAddr = 0b10 << 14,
    Read = 0b11 << 14,
}
/// `MdioBus` trait
/// Driver needs to implement the Clause 22
/// Optional Clause 45 is the device supports this.
///
/// Claus 45 methodes are bases on <https://www.ieee802.org/3/efm/public/nov02/oam/pannell_oam_1_1102.pdf>
pub trait MdioBus {
    type Error;

    /// Read, Clause 22
    async fn read_cl22(&mut self, phy_id: PhyAddr, reg: RegC22) -> Result<RegVal, Self::Error>;

    /// Write, Clause 22
    async fn write_cl22(&mut self, phy_id: PhyAddr, reg: RegC22, reg_val: RegVal) -> Result<(), Self::Error>;

    /// Read, Clause 45
    /// This is the default implementation.
    /// Many hardware these days support direct Clause 45 operations.
    /// Implement this function when your hardware supports it.
    async fn read_cl45(&mut self, phy_id: PhyAddr, regc45: (u8, RegC45)) -> Result<RegVal, Self::Error> {
        // Write FN
        let val = (Reg13Op::Addr as RegVal) | RegVal::from(regc45.0 & DEV_MASK);

        self.write_cl22(phy_id, REG13, val).await?;
        // Write Addr
        self.write_cl22(phy_id, REG14, regc45.1).await?;

        // Write FN
        let val = (Reg13Op::Read as RegVal) | RegVal::from(regc45.0 & DEV_MASK);
        self.write_cl22(phy_id, REG13, val).await?;
        // Write Addr
        self.read_cl22(phy_id, REG14).await
    }

    /// Write, Clause 45
    /// This is the default implementation.
    /// Many hardware these days support direct Clause 45 operations.
    /// Implement this function when your hardware supports it.
    async fn write_cl45(&mut self, phy_id: PhyAddr, regc45: (u8, RegC45), reg_val: RegVal) -> Result<(), Self::Error> {
        let dev_addr = RegVal::from(regc45.0 & DEV_MASK);
        let reg = regc45.1;

        // Write FN
        let val = (Reg13Op::Addr as RegVal) | dev_addr;
        self.write_cl22(phy_id, REG13, val).await?;
        // Write Addr
        self.write_cl22(phy_id, REG14, reg).await?;

        // Write FN
        let val = (Reg13Op::Write as RegVal) | dev_addr;
        self.write_cl22(phy_id, REG13, val).await?;
        // Write Addr
        self.write_cl22(phy_id, REG14, reg_val).await
    }
}

// #[cfg(test)]
// mod tests {
//     use core::convert::Infallible;

//     use super::{MdioBus, PhyAddr, RegC22, RegVal};

//     #[derive(Debug, PartialEq, Eq)]
//     enum A {
//         Read(PhyAddr, RegC22),
//         Write(PhyAddr, RegC22, RegVal),
//     }

//     struct MockMdioBus(Vec<A>);

//     impl MockMdioBus {
//         pub fn clear(&mut self) {
//             self.0.clear();
//         }
//     }

//     impl MdioBus for MockMdioBus {
//         type Error = Infallible;

//         fn write_cl22(
//             &mut self,
//             phy_id: super::PhyAddr,
//             reg: super::RegC22,
//             reg_val: super::RegVal,
//         ) -> Result<(), Self::Error> {
//             self.0.push(A::Write(phy_id, reg, reg_val));
//             Ok(())
//         }

//         fn read_cl22(
//             &mut self,
//             phy_id: super::PhyAddr,
//             reg: super::RegC22,
//         ) -> Result<super::RegVal, Self::Error> {
//             self.0.push(A::Read(phy_id, reg));
//             Ok(0)
//         }
//     }

//     #[test]
//     fn read_test() {
//         let mut mdiobus = MockMdioBus(Vec::with_capacity(20));

//         mdiobus.clear();
//         mdiobus.read_cl22(0x01, 0x00).unwrap();
//         assert_eq!(mdiobus.0, vec![A::Read(0x01, 0x00)]);

//         mdiobus.clear();
//         mdiobus.read_cl45(0x01, (0xBB, 0x1234)).unwrap();
//         assert_eq!(
//             mdiobus.0,
//             vec![
//                 #[allow(clippy::identity_op)]
//                 A::Write(0x01, 13, (0b00 << 14) | 27),
//                 A::Write(0x01, 14, 0x1234),
//                 A::Write(0x01, 13, (0b11 << 14) | 27),
//                 A::Read(0x01, 14)
//             ]
//         );
//     }

//     #[test]
//     fn write_test() {
//         let mut mdiobus = MockMdioBus(Vec::with_capacity(20));

//         mdiobus.clear();
//         mdiobus.write_cl22(0x01, 0x00, 0xABCD).unwrap();
//         assert_eq!(mdiobus.0, vec![A::Write(0x01, 0x00, 0xABCD)]);

//         mdiobus.clear();
//         mdiobus.write_cl45(0x01, (0xBB, 0x1234), 0xABCD).unwrap();
//         assert_eq!(
//             mdiobus.0,
//             vec![
//                 A::Write(0x01, 13, 27),
//                 A::Write(0x01, 14, 0x1234),
//                 A::Write(0x01, 13, (0b01 << 14) | 27),
//                 A::Write(0x01, 14, 0xABCD)
//             ]
//         );
//     }
// }
