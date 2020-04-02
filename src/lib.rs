#![no_std]
#![forbid(unsafe_code)]

extern crate embedded_hal as hal;

#[macro_use]
extern crate bitflags;

use hal::blocking::i2c::{Write, WriteRead};

pub const ADDRESS: u8 = 0x6B;

macro_rules! registers {
    ($
        (
            $(#[$outer:meta])*
            $registerName: ident ($registerAddress: pat) { 
                #[$bit7meta:meta]
                $bit7: ident,
                #[$bit6meta:meta]
                $bit6: ident,
                #[$bit5meta:meta]
                $bit5: ident,
                #[$bit4meta:meta]
                $bit4: ident,
                #[$bit3meta:meta]
                $bit3: ident,
                #[$bit2meta:meta]
                $bit2: ident,
                #[$bit1meta:meta]
                $bit1: ident,
                #[$bit0meta:meta]
                $bit0: ident,
                Default { $($default:ident),* }
            }
        ),
    *) => {
        paste::item!{
            /// BQ24195 state, as viewed from I2C
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

                    /// Read the state of a single register over I2C, updating the chip state.
                    /// If an error occurs, the chip state remains the same.
                    pub fn [<read_$registerName:snake:lower>]<E, I2C: WriteRead<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
                        let mut value = [0u8; 1];
                        i2c.write_read(ADDRESS, &[$registerAddress], &mut value)?;
                        self.[<$registerName:snake:lower>] = value[0].into();
                        Ok(())
                    }

                    /// Write the state of a single register over I2C, updating the chip state.
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
                    #[$bit7meta]
                    const $bit7 = 0b1000_0000;
                    #[$bit6meta]
                    const $bit6 = 0b0100_0000;
                    #[$bit5meta]
                    const $bit5 = 0b0010_0000;
                    #[$bit4meta]
                    const $bit4 = 0b0001_0000;
                    #[$bit3meta]
                    const $bit3 = 0b0000_1000;
                    #[$bit2meta]
                    const $bit2 = 0b0000_0100;
                    #[$bit1meta]
                    const $bit1 = 0b0000_0010;
                    #[$bit0meta]
                    const $bit0 = 0b0000_0001;
                }
            }

            impl Default for $registerName {
                fn default() -> $registerName {
                    $($registerName::$default |)* $registerName {
                        bits: 0u8
                    }
                }
            }

            impl From<u8> for $registerName {
                fn from(bits: u8) -> $registerName {
                    $registerName {
                        bits
                    }
                }
            }

            impl Into<u8> for $registerName {
                fn into(self) -> u8 {
                    self.bits
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
        /// Buck Converter Control (0 = Restart Buck Converter, 1 = Buck Converter Stops, system l oad supplied by battery)
        EN_HIZ,
        /// Input Voltage Limit Offset Bit 3: 640mV
        VINDPM_3,
        /// Input Voltage Limit Offset Bit 2: 320 mV
        VINDPM_2,
        /// Input Voltage Limit Offset Bit 1: 160 mV
        VINDPM_1,
        /// Input Voltage Limit Offset Bit 0: 80 mV
        VINDPM_0,
        /// Input Current Limit Bit 2
        IINLIM_2,
        /// Input Current Limit Bit 1
        IINLIM_1,
        /// Input Current Limit Bit 0
        IINLIM_0,
    Default { VINDPM_2, VINDPM_1 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A586%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    PowerOnConfiguration (0x01) {
        /// Resets this register upon writing this value, returns to 0 after reset
        REGISTER_RESET,
        /// Reset I2C watchdog timer, returns to 0 after reset
        I2C_WATCHDOG_TIMER_RESET,
        /// Charger Configuration Bit 1 (10/11 = OTG)
        CHG_CONFIG_1,
        /// Charger Configuration Bit 0 (00 = Charge Disable, 01 = Charge Battery)
        CHG_CONFIG_0,
        /// Minimum System Voltage Limit Offset Bit 2: 0.4V
        SYS_MIN_2,
        /// Minimum System Voltage Limit Offset Bit 2: 0.2V
        SYS_MIN_1,
        /// Minimum System Voltage Limit Offset Bit 2: 0.1V
        SYS_MIN_0,
        /// Reserved, must write 1
        RESERVED,
    Default {
        CHG_CONFIG_0,
        SYS_MIN_2,
        SYS_MIN_0,
        RESERVED
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A158%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ChargeCurrentControl (0x02) {
        /// Fast Charge Current Limit Offset Bit 5: 2048 mA
        ICHG_5,
        /// Fast Charge Current Limit Offset Bit 4: 1024 mA
        ICHG_4,
        /// Fast Charge Current Limit Offset Bit 3: 512 mA
        ICHG_3,
        /// Fast Charge Current Limit Offset Bit 2: 256 mA
        ICHG_2,
        /// Fast Charge Current Limit Offset Bit 1: 128 mA
        ICHG_1,
        /// Fast Charge Current Limit Offset Bit 0: 64 mA
        ICHG_0,
        /// Reserved, must write 0
        RESERVED,
        /// Force 20% of fast-charge current limit and 500% of pre-charge current limit
        FORCE_20PCT,
    Default { ICHG_4, ICHG_3 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A158%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C409.9%2C0%5D)
    PreChargeTerminationCurrentControl (0x03) {
        /// Pre-Charge Current Limit Offset Bit 3: 1024 mA
        IPRECHG_3,
        /// Pre-Charge Current Limit Offset Bit 2: 512 mA
        IPRECHG_2,
        /// Pre-Charge Current Limit Offset Bit 1: 256 mA
        IPRECHG_1,
        /// Pre-Charge Current Limit Offset Bit 0: 128 mA
        IPRECHG_0,
        /// Termination Current Limit Offset Bit 3: 1024 mA
        ITERM_3,
        /// Termination Current Limit Offset Bit 2: 512 mA
        ITERM_2,
        /// Termination Current Limit Offset Bit 1: 256 mA
        ITERM_1,
        /// Termination Current Limit Offset Bit 0: 128 mA
        ITERM_0,
    Default { IPRECHG_0, ITERM_0 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A601%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ChargeVoltageControl (0x04) {
        /// Charger Voltage Limit Offset Bit 5: 512mV
        VREG_5,
        /// Charger Voltage Limit Offset Bit 4: 256mV
        VREG_4,
        /// Charger Voltage Limit Offset Bit 3: 128mV
        VREG_3,
        /// Charger Voltage Limit Offset Bit 2: 64mV
        VREG_2,
        /// Charger Voltage Limit Offset Bit 1: 32mV
        VREG_1,
        /// Charger Voltage Limit Offset Bit 0: 16mV
        VREG_0,
        /// Battery Precharge to Fast Charge Threshold: (0 = 2.8V, 1 = 3.0V)
        BATLOWV,
        /// Battery Recharge to Threshold (below battery regulation voltage) (0 = 100 mV, 1 = 300 mV)
        VRECHG,
    Default {
        VREG_5,
        VREG_3,
        VREG_2,
        BATLOWV
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A601%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C437.5%2C0%5D)
    ChargeTerminationTimerControlRegister (0x05) {
        /// Charging Termination Enable
        EN_TERM,
        /// Termination Indicator Threshold
        TERM_STAT,
        /// I2C Watchdog Timer Setting Bit 1: (10 = 80s, 11 = 160s)
        WATCHDOG_1,
        /// I2C Watchdog Timer Setting Bit 0: (00 = Disable Timer, 01 = 40s)
        WATCHDOG_0,
        /// Charging Safety Timer Enable
        EN_TIMER,
        /// Fast Charge Timer Setting Bit 1 (10 = 12h, 11 = 20h)
        CHG_TIMER_1,
        /// Fast Charge Timer Setting Bit 0 (00 = 5h, 01 = 8h)
        CHG_TIMER_0,
        /// Reserved, must write 0
        RESERVED,
    Default {
        EN_TERM,
        WATCHDOG_0,
        EN_TIMER,
        CHG_TIMER_0
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A609%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ThermalRegulationControl (0x06) {
        /// Reserved, must write 0
        RESERVED_7,
        /// Reserved, must write 0
        RESERVED_6,
        /// Reserved, must write 0
        RESERVED_5,
        /// Reserved, must write 0
        RESERVED_4,
        /// Reserved, must write 0
        RESERVED_3,
        /// Reserved, must write 0
        RESERVED_2,
        /// Thermal Regulation Threshold Bit 1 (10 = 100C, 11 = 120C)
        TREG_1,
        /// Thermal Regulation Threshold Bit 0 (00 = 60C, 01 = 80C)
        TREG_0,
    Default { TREG_1, TREG_0 }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A609%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C463.9%2C0%5D)
    MiscOperationControl (0x07) {
        /// Force DPDM detection
        DPDM_EN,
        /// Safety Timer Setting during Input DPM and Thermal Regulation (1 = safety timer slowed by 2x, 0 = normal speed)
        TMR2X_EN,
        /// Force BATFET Off
        BATFET_DISABLE,
        /// Reserved, must write 0
        RESERVED_4,
        /// Reserved, must write 1
        RESERVED_3,
        /// Reserved, must write 0
        RESERVED_2,
        /// Interrupt Mask Bit 1 (1 = interrupt on CHRG_FAULT)
        INT_MASK_1,
        /// Interrupt Mask Bit 0 (0 = interrupt on BAT_FAULT)
        INT_MASK_0,
    Default {
        TMR2X_EN,
        RESERVED_3,
        INT_MASK_1,
        INT_MASK_0
    }},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A618%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    SystemStatus (0x08) {
        /// VBUS Status Bit 1 (10 = Adapter port, 11 = OTG)
        VBUS_STAT_1,
        /// VBUS Status Bit 0 (00 = Unknown, 01 = USB host)
        VBUS_STAT_0,
        /// Charging Status Bit 1 (10 = Fast Charging, 11 = Charge Termination Done)
        CHRG_STAT_1,
        /// Charging Status Bit 0 (00 = Not Charging, 01 = Pre-Charge)
        CHRG_STAT_0,
        /// DPM Status (0 = Not DPM, 1 = VINDPM / IINDPM)
        DPM_STAT,
        /// Power Good Status (0 = not good, 1 = good)
        PG_STAT,
        /// Thermal Regulation Status (0 = normal, 1 = thermal regulation)
        THERM_STAT,
        /// VSYSMIN Regulation Status (0 = BAT > VSYSTMIN, 1 = BAT < VSYSMIN)
        VSYS_STAT,
    Default {}},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A618%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C468.1%2C0%5D)
    Fault (0x09) {
        /// Watchdog Fault Status (0 = normal, 1 = watchdog timer expired)
        WATCHDOG_FAULT,
        /// Reserved, always 0
        RESERVED,
        /// Charging Fault Bit 1 (10 = Thermal shutdown, 11 = Charge Safety Timer Expiration)
        CHRG_FAULT_1,
        /// Charging Fault Bit 0 (00 = Normal, 01 = Input Fault (VBUS OVP or VBAT < VBUS < 3.8V))
        CHRG_FAULT_0,
        /// Battery Fault: (0 = Normal, 1 = BATOVP)
        BAT_FAULT,
        /// NTC Fault Bit 2 (110 = Hot, 101 = Cold)
        NTC_FAULT_2,
        /// NTC Fault Bit 1 (110 = Hot)
        NTC_FAULT_1,
        /// NTC Fault Bit 0 (000 = Normal, 101 = Cold)
        NTC_FAULT_0,
    Default {}},
    /// [Relevant BQ24195 Datasheet Section](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A626%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    VendorPartRevisionStatus (0x0A) {
        /// Reserved, always 0
        RESERVED_7,
        /// Reserved, always 0
        RESERVED_6,
        /// Always 1
        PN_2,
        /// Always 0
        PN_1,
        /// Always 0
        PN_0,
        /// ???
        TS_PROFILE,
        /// Always 1
        DEV_REG_0,
        /// Always 1
        DEV_REG_1,
    Default {
        PN_2,
        DEV_REG_0,
        DEV_REG_1
    }}
);
