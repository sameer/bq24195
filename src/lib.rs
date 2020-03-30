#![no_std]
#![forbid(unsafe_code)]

extern crate embedded_hal as hal;

#[macro_use]
extern crate bitflags;

use hal::blocking::i2c::{Read, Write, WriteRead};

pub struct Bq24195<I2C> {
    i2c: I2C,
    state: State,
}

impl<I2C, E> Bq24195<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E> + Read<Error = E>,
{
    const ADDRESS: u8 = 0x6B;

    pub fn try_new(mut i2c: I2C) -> Result<Self, E> {
        let mut bq24195 = Self {
            i2c,
            state: State::default(),
        };
        // Multi-read of current status
        bq24195.i2c.write(Self::ADDRESS, &[0x08])?;
        let mut values = [0u8; 2];
        bq24195.i2c.read(Self::ADDRESS, &mut values)?;
        bq24195.state.system_status = values[0].into();
        bq24195.state.fault = values[1].into();

        Ok(bq24195)
    }
}

#[derive(Default)]
struct State {
    input_source_control: InputSourceControl,
    power_on_configuration: PowerOnConfiguration,
    charge_current_control: ChargeCurrentControl,
    pre_charge_termination_current_control: PreChargeTerminationCurrentControl,
    charge_voltage_control: ChargeVoltageControl,
    charge_termination_timer_control: ChargeTerminationTimerControlRegister,
    thermal_regulation_control: ThermalRegulationControl,
    misc_operation_control: MiscOperationControl,
    system_status: SystemStatus,
    fault: Fault,
    vendor_part_revision_status: VendorPartRevisionStatus,
}

macro_rules! register {
    ($registerName: ident { $bit7: ident, $bit6: ident, $bit5: ident, $bit4: ident, $bit3: ident, $bit2: ident, $bit1: ident, $bit0: ident }, Default { $($default:ident), * }) => {
        bitflags! {
            struct $registerName: u8 {
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
            impl Into<u8> for & $registerName {
                fn into(self) -> u8 {
                    self.bits
                }
            }
        }
    };
}

register!(
    InputSourceControl {
        EN_HIZ,
        VINDPM_3,
        VINDPM_2,
        VINDPM_1,
        VINDPM_0,
        IINLIM_2,
        IINLIM_1,
        IINLIM_0
    },
    Default { VINDPM_2, VINDPM_1 }
);

register!(
    PowerOnConfiguration {
        REGISTER_RESET,
        I2C_WATCHDOG_TIMER_RESET,
        CHG_CONFIG_1,
        CHG_CONFIG_0,
        SYS_MIN_2,
        SYS_MIN_1,
        SYS_MIN_0,
        RESERVED
    },
    Default {
        CHG_CONFIG_0,
        SYS_MIN_2,
        SYS_MIN_0,
        RESERVED
    }
);

register!(
    ChargeCurrentControl {
        ICHG_5,
        ICHG_4,
        ICHG_3,
        ICHG_2,
        ICHG_1,
        ICHG_0,
        RESERVED,
        FORCE_20PCT
    },
    Default { ICHG_4, ICHG_3 }
);

register!(
    PreChargeTerminationCurrentControl {
        IPRECHG_3,
        IPRECHG_2,
        IPRECHG_1,
        IPRECHG_0,
        ITERM_3,
        ITERM_2,
        ITERM_1,
        ITERM_0
    },
    Default { IPRECHG_0, ITERM_0 }
);

register!(
    ChargeVoltageControl {
        VREG_5,
        VREG_4,
        VREG_3,
        VREG_2,
        VREG_1,
        VREG_0,
        BATLOWV,
        VRECHG
    },
    Default {
        VREG_5,
        VREG_3,
        VREG_2,
        BATLOWV
    }
);

register!(
    ChargeTerminationTimerControlRegister {
        EN_TERM,
        TERM_STAT,
        WATCHDOG_1,
        WATCHDOG_0,
        EN_TIMER,
        CHG_TIMER_1,
        CHG_TIMER_0,
        RESERVED
    },
    Default {
        EN_TERM,
        WATCHDOG_0,
        EN_TIMER,
        CHG_TIMER_0
    }
);

register!(
    ThermalRegulationControl {
        RESERVED_7,
        RESERVED_6,
        RESERVED_5,
        RESERVED_4,
        RESERVED_3,
        RESERVED_2,
        TREG_1,
        TREG_0
    },
    Default { TREG_1, TREG_0 }
);

register!(
    MiscOperationControl {
        DPDM_EN,
        TMR2X_EN,
        BATFET_DISABLE,
        RESERVED_4,
        RESERVED_3,
        RESERVED_2,
        INT_MASK_1,
        INT_MASK_0
    },
    Default {
        TMR2X_EN,
        RESERVED_3,
        INT_MASK_1,
        INT_MASK_0
    }
);

register!(
    SystemStatus {
        VBUS_STAT_1,
        VBUS_STAT_0,
        CHRG_STAT_1,
        CHRG_STAT_0,
        DPM_STAT,
        PG_STAT,
        THERM_STAT,
        VSYS_STAT
    },
    Default {}
);

register!(
    Fault {
        WATCHDOG_FAULT,
        RESERVED,
        CHRG_FAULT_1,
        CHRG_FAULT_0,
        BAT_FAULT,
        NTC_FAULT_2,
        NTC_FAULT_1,
        NTC_FAULT_0
    },
    Default {}
);

register!(
    VendorPartRevisionStatus {
        RESERVED_7,
        RESERVED_6,
        PN_2,
        PN_1,
        PN_0,
        TS_PROFILE,
        DEV_REG_0,
        DEV_REG_1
    },
    Default {
        PN_2,
        DEV_REG_0,
        DEV_REG_1
    }
);
