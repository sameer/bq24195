//! # BQ24195
//! BQ24195 is a single cell charger intended for use in portable devices. The features of the charger are summarized as they pertain to this library below.
//!
//! ## Functional Modes
//!
//! Chip mode is indicated by [`Fault::WATCHDOG_FAULT`](struct.Fault.html#associatedconstant.WATCHDOG_FAULT), 1 = Default Mode and 0 = Host Mode.
//!
//! ### Default Mode
//!
//! By default, the chip does not require host management and can be used as an autonomous charger. In this case, I2C can be used to just monitor the chip's status.
//!
//! ### Host Mode
//!
//! In host mode,
//! The chip will transition to host mode upon writing to any chip register, resetting the watchdog timer.
//! If the watchdog timer set in [`ChargeTerminationTimerControl::WATCHDOG[1:0]`](struct.ChargeTerminationTimerControl.html#associatedconstant.WATCHDOG_1) (default 40s) expires, the chip transitions back to default mode.
//! To stay in host mode, either write 1 twice to [`PowerOnConfiguration::I2C_WATCHDOG_TIMER_RESET`](struct.PowerOnConfiguration.html#associatedconstant.I2C_WATCHDOG_TIMER_RESET) to reset the timer before it expires,
//! or disable the timer entirely by setting [`ChargeTerminationTimerControl::WATCHDOG[1:0]`](struct.ChargeTerminationTimerControl.html#associatedconstant.WATCHDOG_1) to 00.
//!
//! ## BATFET
//!
//! The battery field-effect transistor (BATFET) is used to control the flow of current to the battery.
//! You can manually disable it by writing 1 to [`MiscOperationControl::BATFET_DISABLE`](struct.MiscOperationControl.html#associatedconstant.BATFET_DISABLE). This disconnects the battery, disabling both charging and discharging.
//!
//! ## Power Path Management
//! [Dynamic Power Management](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A326%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C202.4%2C0%5D) ensures compliance with the USB specification.
//! It continuously monitors the input current and input voltage to maintain nominal system performance.
//!
//! ### Overloaded input source
//! If input current exceeds [`InputSourceControl::IINLIM[2:0]`](struct.InputSourceControl.html#associatedconstant.IINLIM_2) or input voltage falls below 3.88V + [`InputSourceControl::VINDPM[3:0]`](struct.InputSourceControl.html#associatedconstant.VINDPM_3),
//! then the input source is considered overloaded. [`SystemStatus::DPM_STATUS`](struct.SystemStatus.html#associatedconstant.DPM_STAT) will indicate these conditions.
//! DPM will reduce charge current until it is no longer overloaded.
//!
//! For example, the default input voltage limit offset is set to 480 mV (3.88V + 480mV = 4.36V), which is the correct voltage for fully charging a Lithium-Ion cell.
//!
//! ### Supplement mode
//!
//! If the input source is still overloaded when the charge current is dropped to 0A, the chip enters supplement mode; the BATFET is turned on and the battery begins discharging to supplement the input source.
//!
//! ## Battery Charging Management
//!
//! BQ24195 is designed to charge a single-cell Li-Ion high capacity battery (i.e. 18650).
//!
//! Although the default current limit [`InputSourceControl::IINLIM[2:0]`](struct.InputSourceControl.html#associatedconstant.IINLIM_2) is listed as 100 mA, the actual default value will depend on USB detection.
//!
//! ### USB D+/D- Detection
//!
//!  The charger detects the type of USB connection and sets the input current limit to comply with the USB specification:
//!
//! * Floating data lines after 500ms: 100mA (IINLIM[2:0] = 000)
//! * Standard Down Stream Port (SDP) (connected to USB Host)
//!     * OTG pin = 0 : 100 mA (IINLIM[2:0] = 000)
//!     * OTG pin = 1 : 500 mA (IINLIM[2:0] = 010)
//! * Charging Down Stream Port or Dedicated Charging Down Stream Port (CDP/DCP): 1.5A (IINLIM[2:0] = 101)
//!
//! ### HIZ
//!
//! When the chip is in high-impedance (HIZ) the buck converter is disabled and the system load is supplied by the battery. To comply with the USB battery charging specification, the chip enters HIZ if the input source is a 100mA USB host and the battery voltage is above VBATGD (3.55V).
//!
//! This can be manually controlled via [InputSourceControl::EN_HIZ](struct.InputSourceControl.html#associatedconstant.EN_HIZ).
//!
//! ### ILIM
//!
//! ILIM is a hardware pin for a limiting maximum input current. It is grounded with a resistor using the following formula: `ILIM = 1V / R_ILIM * 530`
//!
//! By changing [`InputSourceControl::IINLIM[2:0]`](struct.InputSourceControl.html#associatedconstant.IINLIM_2), you can [ONLY REDUCE the input current limit below ILIM](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A436%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C481.2%2C0%5D).
//!
//! ### Charging Profile
//!
//! To safely charge the battery and preserve its lifetime, charging is split into several phases (collectively referred to as the charging profile):
//!
//! * Almost Empty or Empty Cell: the battery voltage is below the battery short voltage (VBAT_SHORT = 2V) and is charged at 100mA, which cannot be changed
//!     * [Lithium-Ion batteries should never be drained below 3V](https://electronics.stackexchange.com/questions/219222/is-draining-a-li-ion-to-2-5v-harmful-to-the-battery). If they are, battery life is significantly reduced.
//! * Pre-charge: the battery voltage is 2V to 3V, and current limit is set to 128mA + [`PreChargeTerminationCurrentControl::IPRECHG[3:0]`](struct.PreChargeTerminationCurrentControl.html#associatedconstant.IPRECHG_3)
//! * Fast Charge: the battery voltage is above [`ChargeVoltageControl::BATLOWV`](struct.ChargeVoltageControl.html#associatedconstant.BATLOWV) (2.8V/3V), and the current limit is set to 512mA + [`ChargeCurrentControl::ICHG[5:0]`](struct.ChargeCurrentControl.html#associatedconstant.ICHG_5)
//! * Constant-Voltage: the battery voltage has reached the recharge threshold voltage (3.504V + [`ChargeVoltageControl::VREG[5:0]`](struct.ChargeVoltageControl.html#associatedconstant.VREG_5)),and charging current drops rapidly to 128mA + [`PreChargeTerminationCurrentControl::ITERM[3:0]`](struct.PreChargeTerminationCurrentControl.html#associatedconstant.ITERM_3) at which charging is terminated
//!
//! ### Battery Temperature
//!
//! An external thermistor is used to measure battery temperature. The reading must be between VLTF and VHTF, else the chip will suspend charging. The nature of the thermal fault will be indicated in [`Fault::NTC_FAULT[2:0]`](struct.Fault.html#associatedconstant.NTC_FAULT_2)
//!
//! In some cases, there is no therimstor present because the battery is external, in which case the chip may always report a normal battery temperature.
//!
//! ### Charging Termination
//!
//! BQ24195 will terminate charging when the battery voltage has reached the recharge threshold and the current has dropped below the termination threshold. [`SystemStatus::CHRG_STAT_1`](struct.SystemStatus.html#associatedconstant.CHRG_STAT_1) will become 11.
//!
//! Termination will be disabled if the device is in thermal regulation or input current/voltage regulation. It can also be disabled manually by writing 0 to [`ChargeTerminationTimerControl::EN_TERM`](struct.ChargeTerminationTimerControl.html#associatedconstant.EN_TERM)
//!
//! When [`ChargeCurrentControl::FORCE_20PCT`](struct.ChargeCurrentControl.html#associatedconstant.FORCE_20PCT) is set, make sure that the termination current [`PreChargeTerminationCurrentControl::ITERM[3:0]`](struct.PreChargeTerminationCurrentControl.html#associatedconstant.ITERM_3) is less than 20% of the charging current, otherwise charging will not terminate.
//!
//! Writing 1 to [`ChargeTerminationTimerControl::TERM_STAT`](struct.ChargeTerminationTimerControl.html#associatedconstant.TERM_STAT) will enable an early charge done indication on the STAT pin when charging current falls below 800 mA.
//!
//! ### Safety Timer
//!
//! A safety timer is used to stop charging if it is taking too long. When connected to a 100mA USB source, it is ALWAYS a maximum of 45 minutes. Otherwise, in device mode, it is 5 hours long. In host mode, it is 8 hours long but can be changed.
//! If battery voltage is below [`ChargeVoltageControl::BATLOWV`](struct.ChargeVoltageControl.html#associatedconstant.BATLOWV), it is 1 hour.
//! An expired safety timer will appear as [`Fault::CHRG_FAULT[1:0]`](struct.Fault.html#associatedconstant.CHRG_FAULT_1) equal to 11. The timer can be enabled/disabled by writing to [`ChargeTerminationTimerControl::EN_TIMER`](struct.ChargeTerminationTimerControl.html#associatedconstant.EN_TIMER).
//!
//! #### Restarting the timer
//!
//! The timer can be restarted by disabling and then re-enabling it. It is also restarted when [`PowerOnConfiguration::CHG_CONFIG[1:0]`](struct.PowerOnConfiguration.html#associatedconstant.CHG_CONFIG_1) is changed from disabled to any enabled mode.
//!
//! #### Changing the timer
//!
//! To change the timer, the datasheet recommends you first disable it, write the desired value to [`ChargeTerminationTimerControl::CHG_TIMER[2:1]`](struct.ChargeTerminationTimerControl.html#associatedconstant.CHG_TIMER_1), then re-enable it.
//!
//! #### Half clock rate
//!
//! The safety timer will count at half the normal clock rate when in thermal regulation, input voltage/current regulation, or [`ChargeCurrentControl::FORCE_20PCT`](struct.ChargeCurrentControl.html#associatedconstant.FORCE_20PCT) is set.
//! A 5 hour safety timer would actually be 10 hours long.
//!
//! ## Protections
//!
//! ### ILIM
//!
//! Already discussed above.
//!
//! ### Battery Over-Current Protection
//!
//! If the battery voltage is at least 4% above the regulation voltage, charging is immediately disabled and [`Fault::CHRG_FAULT_1`](struct.Fault.html#associatedconstant.CHRG_FAULT_1) goes high.
//!
//! ### Input Over-Voltage
//!
//! An input voltage of over 18V for VBUS will stop buck mode operation and [`Fault::CHRG_FAULT[1:0]`](struct.Fault.html#associatedconstant.CHRG_FAULT_1) will be set to 01.

#![no_std]
#![forbid(unsafe_code)]

extern crate embedded_hal as hal;

use hal::blocking::i2c::{Write, WriteRead};

/// I2C Address of BQ24195
pub const ADDRESS: u8 = 0x6B;

macro_rules! registers {
    ($(
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
        ),*
    ) => {
        paste::item!{
            /// BQ24195 state, as viewed from I2C
            #[derive(Clone)]
            pub struct ChargerState {
                $(
                    [<$registerName:snake:lower>]: $registerName,
                )*
            }
        }

        paste::item! {
            impl ChargerState {
                /// Create a new `ChargerState` struct by reading all registers over I2C
                ///
                /// `Default` is NOT implemented for `ChargerState` because some registers do not actually have a default
                pub fn try_new<E, I2C: WriteRead<Error = E>>(i2c: &mut I2C) -> Result<Self, E> {
                    let mut state = Self {
                        $(
                            [<$registerName:snake:lower>]: $registerName::default(),
                        )*
                    };
                    state.read_all(i2c)?;
                    Ok(state)
                }

                /// Read all registers to set the current state of BQ24195.
                pub fn read_all<E, I2C: WriteRead<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
                    let mut values = [0u8; NUM_REGISTERS];
                    i2c.write_read(ADDRESS, &[0x00], &mut values)?;
                    $(
                        self.[<$registerName:snake:lower>] = values[$registerAddress].into();
                    )*
                    Ok(())
                }

                /// Write chip state to all registers. Useful for taking a preset chip state and applying it.
                ///
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
                    ///
                    /// If an error occurs, the chip state remains the same.
                    pub fn [<read_$registerName:snake:lower>]<E, I2C: WriteRead<Error = E>>(&mut self, i2c: &mut I2C) -> Result<(), E> {
                        let mut value = [0u8; 1];
                        i2c.write_read(ADDRESS, &[$registerAddress], &mut value)?;
                        self.[<$registerName:snake:lower>] = value[0].into();
                        Ok(())
                    }

                    /// Write the state of a single register over I2C, updating the chip state.
                    ///
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
            $(#[$outer])*
            #[derive(Copy, Debug, PartialEq, Clone, Eq)]
            pub struct $registerName {
                bits: u8
            }

            impl $registerName {
                #[$bit7meta]
                pub const $bit7: Self = Self { bits: 1u8 << 7 };
                #[$bit6meta]
                pub const $bit6: Self = Self { bits: 1u8 << 6 };
                #[$bit5meta]
                pub const $bit5: Self = Self { bits: 1u8 << 5 };
                #[$bit4meta]
                pub const $bit4: Self = Self { bits: 1u8 << 4 };
                #[$bit3meta]
                pub const $bit3: Self = Self { bits: 1u8 << 3 };
                #[$bit2meta]
                pub const $bit2: Self = Self { bits: 1u8 << 2 };
                #[$bit1meta]
                pub const $bit1: Self = Self { bits: 1u8 << 1 };
                #[$bit0meta]
                pub const $bit0: Self = Self { bits: 1u8 << 0 };
            }

            impl core::ops::BitOr for $registerName {
                type Output = Self;
                fn bitor(self, rhs: Self) -> Self {
                    Self { bits: self.bits | rhs.bits }
                }
            }

            impl core::ops::BitOrAssign for $registerName {
                fn bitor_assign(&mut self, rhs: Self) {
                    self.bits |= rhs.bits;
                }
            }

            impl core::ops::BitAnd for $registerName {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self {
                    Self { bits: self.bits & rhs.bits }
                }
            }

            impl core::ops::BitAndAssign for $registerName {
                fn bitand_assign(&mut self, rhs: Self) {
                    self.bits &= rhs.bits;
                }
            }

            impl core::ops::BitXor for $registerName {
                type Output = Self;
                fn bitxor(self, rhs: Self) -> Self {
                    Self { bits: self.bits ^ rhs.bits }
                }
            }

            impl core::ops::BitXorAssign for $registerName {
                fn bitxor_assign(&mut self, rhs: Self) {
                    self.bits ^= rhs.bits;
                }
            }

            impl Default for $registerName {
                fn default() -> $registerName {
                    $($registerName::$default |)* 0u8.into()
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
    /// [Register 0x00](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A578%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C681.2%2C0%5D)
    ///
    /// VINDPM[3:0] is added to 3.88V
    ///
    /// IINLIM[2:0] is scaled in an odd manner:
    /// ```
    /// 000 = 100  mA
    /// 001 = 150  mA
    /// 010 = 500  mA
    /// 011 = 900  mA
    /// 100 = 1200 mA
    /// 101 = 1500 mA
    /// 110 = 2000 mA
    /// 111 = 3000 mA
    /// ```
    InputSourceControl (0x00) {
        /// Buck Converter Control (0 = Restart Buck Converter, 1 = Buck Converter Stops, system load supplied by battery)
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
    /// [Register 0x01](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A586%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ///
    /// SYS_MIN[2:0] is added to 3.0V
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
    /// [Register 0x02](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A158%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ///
    /// ICHG[5:0] is added to 512mA
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
        /// Force 20% of fast-charge current limit and 50% of pre-charge current limit
        FORCE_20PCT,
    Default { ICHG_4, ICHG_3 }},
    /// [Register 0x03](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A158%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C409.9%2C0%5D)
    ///
    /// IPRECHG[3:0] is added to 128mA
    ///
    /// ITERM[3:0] is added to 128mA
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
    /// [Register 0x04](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A601%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
    ///
    /// VREG[5:0] is added to 3.504V
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
    /// [Register 0x05](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A601%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C437.5%2C0%5D)
    ChargeTerminationTimerControl (0x05) {
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
    /// [Register 0x06](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A609%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
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
    /// [Register 0x07](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A609%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C463.9%2C0%5D)
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
    /// [Register 0x08](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A618%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
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
    /// [Register 0x09](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A618%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C468.1%2C0%5D)
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
    Default { WATCHDOG_FAULT }},
    /// [Register 0x0A](https://www.ti.com/lit/ds/symlink/bq24195l.pdf#%5B%7B%22num%22%3A626%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C0%2C720%2C0%5D)
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
        /// 0 = Cold/Hot Window
        TS_PROFILE,
        /// Always 1 (note the inversed bit order here)
        DEV_REG_0,
        /// Always 1
        DEV_REG_1,
    Default {
        PN_2,
        DEV_REG_0,
        DEV_REG_1
    }}
);
