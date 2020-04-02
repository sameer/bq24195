#![no_std]
#![forbid(unsafe_code)]

extern crate embedded_hal as hal;

#[macro_use]
extern crate bitflags;

use hal::blocking::i2c::{Write, WriteRead};

pub const ADDRESS: u8 = 0x6B;

macro_rules! registers {
    ($($(#[$outer:meta])* $registerName: ident ($registerAddress: pat) { $bit7: ident, $bit6: ident, $bit5: ident, $bit4: ident, $bit3: ident, $bit2: ident, $bit1: ident, $bit0: ident, Default { $($default:ident),* }}),*) => {
        paste::item!{
            #[derive(Default, Clone)]
            pub struct ChargerState {
                $(
                    pub [<$registerName:snake:lower>]: $registerName,
                )*
            }
        }

        paste::item! {
            impl ChargerState {
                /// Read all registers to update the state.
                /// Useful fo checking fault detection and system status registers.
                pub fn read_all<E, I2C: WriteRead<Error = E>>(&mut self, i2c: &mut I2C) -> Result<Self, E> {
                    let mut values = [0u8; NUM_REGISTERS];
                    i2c.write_read(ADDRESS, &[0x00], &mut values)?;
                    Ok(Self {
                        $(
                            [<$registerName:snake:lower>]: values[$registerAddress].into(),
                        )*
                    })
                }

                /// Write chip state to all registers.
                /// Useful for taking a preset chip state and applying it.
                /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A98%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C556.4%2C0%5D)
                pub fn write_all<E, I2C: Write<Error = E>>(&self, i2c: &mut I2C) -> Result<(), E> {
                    i2c.write(ADDRESS, &[0x00])?;
                    let mut values = [0u8; NUM_REGISTERS];
                    $(
                        values[$registerAddress] = self.[<$registerName:snake:lower>].into();
                    )*
                    i2c.write(ADDRESS, &values[..=LAST_WRITABLE_REGISTER])?;
                    Ok(())
                }

                $(
                    /// Get a register state from the current chip state. Does NOT do an I2C call.
                    pub fn [<get_$registerName:snake:lower>](&self) -> $registerName {
                        self.[<$registerName:snake:lower>]
                    }

                    /// Read the state of a single register, updating the chip state.
                    /// If an error occurs, the chip state remains the same.
                    pub fn [<read_$registerName:snake:lower>]<E, I2C: WriteRead<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
                        let mut value = [0u8; 1];
                        i2c.write_read(ADDRESS, &[$registerAddress], &mut value)?;
                        self.[<$registerName:snake:lower>] = value[0].into();
                        Ok(())
                    }

                    /// Write the state of a single register, updating the chip state.
                    /// If an error occurs, the chip state remains the same.
                    pub fn [<write_$registerName:snake:lower>]<E, I2C: Write<Error = E>>(&mut self, i2c: &mut I2C, [<$registerName:snake:lower>]: $registerName) -> Result<(), E> {
                        i2c.write(ADDRESS, &[$registerAddress])?;
                        let value = [[<$registerName:snake:lower>].into(); 1];
                        i2c.write(ADDRESS, &value)?;
                        self.[<$registerName:snake:lower>] = [<$registerName:snake:lower>];
                        Ok(())
                    }
                )*
            }
        }

        $(
            bitflags! {
                $(#[$outer])*
                pub struct $registerName: u8 {
                    const $bit7 = 0b1000_0000;
                    const $bit6 = 0b0100_0000;
                    const $bit5 = 0b0010_0000;
                    const $bit4 = 0b0001_0000;
                    const $bit3 = 0b0000_1000;
                    const $bit2 = 0b0000_0100;
                    const $bit1 = 0b0000_0010;
                    const $bit0 = 0b0000_0001;
                }
            }

            paste::item! {
                impl Default for $registerName {
                    fn default() -> $registerName {
                        $($registerName::$default |)* $registerName {
                            bits: 0u8
                        }
                    }
                }
            }

            paste::item! {
                impl From<u8> for $registerName {
                    fn from(bits: u8) -> $registerName {
                        $registerName {
                            bits
                        }
                    }
                }
            }

            paste::item! {
                impl Into<u8> for $registerName {
                    fn into(self) -> u8 {
                        self.bits
                    }
                }
            }
        )*
    };
}

const NUM_REGISTERS: usize = 11;
const LAST_WRITABLE_REGISTER: usize = 7;

registers!(
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A578%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C681.2%2C0%5D)
    InputSourceControl (0x00) {
        EN_HIZ,
        VINDPM_3,
        VINDPM_2,
        VINDPM_1,
        VINDPM_0,
        IINLIM_2,
        IINLIM_1,
        IINLIM_0,
    Default { VINDPM_2, VINDPM_1 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A586%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    PowerOnConfiguration (0x01) {
        REGISTER_RESET,
        I2C_WATCHDOG_TIMER_RESET,
        CHG_CONFIG_1,
        CHG_CONFIG_0,
        SYS_MIN_2,
        SYS_MIN_1,
        SYS_MIN_0,
        RESERVED,
    Default {
        CHG_CONFIG_0,
        SYS_MIN_2,
        SYS_MIN_0,
        RESERVED
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A158%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ChargeCurrentControl (0x02) {
        ICHG_5,
        ICHG_4,
        ICHG_3,
        ICHG_2,
        ICHG_1,
        ICHG_0,
        RESERVED,
        FORCE_20PCT,
    Default { ICHG_4, ICHG_3 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A158%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C409.9%2C0%5D)
    PreChargeTerminationCurrentControl (0x03) {
        IPRECHG_3,
        IPRECHG_2,
        IPRECHG_1,
        IPRECHG_0,
        ITERM_3,
        ITERM_2,
        ITERM_1,
        ITERM_0,
    Default { IPRECHG_0, ITERM_0 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A601%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ChargeVoltageControl (0x04) {
        VREG_5,
        VREG_4,
        VREG_3,
        VREG_2,
        VREG_1,
        VREG_0,
        BATLOWV,
        VRECHG,
    Default {
        VREG_5,
        VREG_3,
        VREG_2,
        BATLOWV
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A601%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C437.5%2C0%5D)
    ChargeTerminationTimerControlRegister (0x05) {
        EN_TERM,
        TERM_STAT,
        WATCHDOG_1,
        WATCHDOG_0,
        EN_TIMER,
        CHG_TIMER_1,
        CHG_TIMER_0,
        RESERVED,
    Default {
        EN_TERM,
        WATCHDOG_0,
        EN_TIMER,
        CHG_TIMER_0
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A609%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ThermalRegulationControl (0x06) {
        RESERVED_7,
        RESERVED_6,
        RESERVED_5,
        RESERVED_4,
        RESERVED_3,
        RESERVED_2,
        TREG_1,
        TREG_0,
    Default { TREG_1, TREG_0 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A609%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C463.9%2C0%5D)
    MiscOperationControl (0x07) {
        DPDM_EN,
        TMR2X_EN,
        BATFET_DISABLE,
        RESERVED_4,
        RESERVED_3,
        RESERVED_2,
        INT_MASK_1,
        INT_MASK_0,
    Default {
        TMR2X_EN,
        RESERVED_3,
        INT_MASK_1,
        INT_MASK_0
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A618%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    SystemStatus (0x08) {
        VBUS_STAT_1,
        VBUS_STAT_0,
        CHRG_STAT_1,
        CHRG_STAT_0,
        DPM_STAT,
        PG_STAT,
        THERM_STAT,
        VSYS_STAT,
    Default {}},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A618%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C468.1%2C0%5D)
    Fault (0x09) {
        WATCHDOG_FAULT,
        RESERVED,
        CHRG_FAULT_1,
        CHRG_FAULT_0,
        BAT_FAULT,
        NTC_FAULT_2,
        NTC_FAULT_1,
        NTC_FAULT_0,
    Default {}},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A626%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    VendorPartRevisionStatus (0x0A) {
        RESERVED_7,
        RESERVED_6,
        PN_2,
        PN_1,
        PN_0,
        TS_PROFILE,
        DEV_REG_0,
        DEV_REG_1,
    Default {
        PN_2,
        DEV_REG_0,
        DEV_REG_1
    }}
);
