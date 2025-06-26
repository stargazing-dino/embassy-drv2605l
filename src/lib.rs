#![no_std]
#![allow(async_fn_in_trait)]

use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

#[cfg(feature = "defmt")]
use defmt::Format;

pub const DRV2605L_ADDR: u8 = 0x5A;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Error<E> {
    I2c(E),
    InvalidParameter,
    CalibrationFailed,
}

pub mod registers {
    pub const STATUS: u8 = 0x00;
    pub const MODE: u8 = 0x01;
    pub const RTP_INPUT: u8 = 0x02;
    pub const LIBRARY_SELECTION: u8 = 0x03;
    pub const WAVEFORM_SEQUENCER_1: u8 = 0x04;
    pub const WAVEFORM_SEQUENCER_2: u8 = 0x05;
    pub const WAVEFORM_SEQUENCER_3: u8 = 0x06;
    pub const WAVEFORM_SEQUENCER_4: u8 = 0x07;
    pub const WAVEFORM_SEQUENCER_5: u8 = 0x08;
    pub const WAVEFORM_SEQUENCER_6: u8 = 0x09;
    pub const WAVEFORM_SEQUENCER_7: u8 = 0x0A;
    pub const WAVEFORM_SEQUENCER_8: u8 = 0x0B;
    pub const GO: u8 = 0x0C;
    pub const OVERDRIVE_TIME_OFFSET: u8 = 0x0D;
    pub const SUSTAIN_TIME_OFFSET_POS: u8 = 0x0E;
    pub const SUSTAIN_TIME_OFFSET_NEG: u8 = 0x0F;
    pub const BRAKE_TIME_OFFSET: u8 = 0x10;
    pub const AUDIO_TO_VIBE_CONTROL: u8 = 0x11;
    pub const AUDIO_TO_VIBE_MIN_INPUT: u8 = 0x12;
    pub const AUDIO_TO_VIBE_MAX_INPUT: u8 = 0x13;
    pub const AUDIO_TO_VIBE_MIN_OUTPUT: u8 = 0x14;
    pub const AUDIO_TO_VIBE_MAX_OUTPUT: u8 = 0x15;
    pub const RATED_VOLTAGE: u8 = 0x16;
    pub const OVERDRIVE_CLAMP_VOLTAGE: u8 = 0x17;
    pub const AUTO_CALIB_COMP_RESULT: u8 = 0x18;
    pub const AUTO_CALIB_BACK_EMF_RESULT: u8 = 0x19;
    pub const FEEDBACK_CONTROL: u8 = 0x1A;
    pub const CONTROL1: u8 = 0x1B;
    pub const CONTROL2: u8 = 0x1C;
    pub const CONTROL3: u8 = 0x1D;
    pub const CONTROL4: u8 = 0x1E;
    pub const CONTROL5: u8 = 0x1F;
    pub const LRA_LOOP_PERIOD: u8 = 0x20;
    pub const VBAT_VOLTAGE_MONITOR: u8 = 0x21;
    pub const LRA_RESONANCE_PERIOD: u8 = 0x22;
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Mode {
    InternalTrigger = 0x00,
    ExternalTriggerEdge = 0x01,
    ExternalTriggerLevel = 0x02,
    PwmOrAnalogInput = 0x03,
    AudioToVibe = 0x04,
    RealTimePlayback = 0x05,
    Diagnostics = 0x06,
    AutoCalibration = 0x07,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum MotorType {
    ERM,
    LRA,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Library {
    Empty = 0,
    LibraryA = 1,
    LibraryB = 2,
    LibraryC = 3,
    LibraryD = 4,
    LibraryE = 5,
    LRA = 6,
}

pub struct Drv2605l<I2C> {
    i2c: I2C,
    motor_type: MotorType,
}

impl<I2C, E> Drv2605l<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            motor_type: MotorType::LRA,
        }
    }

    pub async fn init(&mut self) -> Result<(), Error<E>> {
        self.reset().await?;
        Timer::after(Duration::from_millis(2)).await;
        
        self.exit_standby().await?;
        
        if self.motor_type == MotorType::LRA {
            self.write_register(registers::FEEDBACK_CONTROL, 0x80).await?;
            self.set_library(Library::LRA).await?;
        } else {
            self.write_register(registers::FEEDBACK_CONTROL, 0x00).await?;
            self.set_library(Library::LibraryB).await?;
        }
        
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::MODE, 0x80).await
    }

    pub async fn exit_standby(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::MODE, 0x00).await
    }

    pub async fn enter_standby(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::MODE, 0x40).await
    }

    pub async fn set_mode(&mut self, mode: Mode) -> Result<(), Error<E>> {
        let current = self.read_register(registers::MODE).await?;
        let new_value = (current & 0xF8) | (mode as u8);
        self.write_register(registers::MODE, new_value).await
    }

    pub async fn set_library(&mut self, library: Library) -> Result<(), Error<E>> {
        self.write_register(registers::LIBRARY_SELECTION, library as u8).await
    }

    pub async fn set_motor_type(&mut self, motor_type: MotorType) -> Result<(), Error<E>> {
        self.motor_type = motor_type;
        
        match motor_type {
            MotorType::LRA => {
                self.write_register(registers::FEEDBACK_CONTROL, 0x80).await?;
                self.set_library(Library::LRA).await
            }
            MotorType::ERM => {
                self.write_register(registers::FEEDBACK_CONTROL, 0x00).await?;
                self.set_library(Library::LibraryB).await
            }
        }
    }

    pub async fn go(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::GO, 0x01).await
    }

    pub async fn stop(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::GO, 0x00).await
    }

    pub async fn is_playing(&mut self) -> Result<bool, Error<E>> {
        let go_reg = self.read_register(registers::GO).await?;
        Ok(go_reg & 0x01 != 0)
    }

    pub async fn set_waveform(&mut self, slot: u8, effect: u8) -> Result<(), Error<E>> {
        if slot > 7 {
            return Err(Error::InvalidParameter);
        }
        
        let reg = registers::WAVEFORM_SEQUENCER_1 + slot;
        self.write_register(reg, effect).await
    }

    pub async fn clear_waveform_sequence(&mut self) -> Result<(), Error<E>> {
        for i in 0..8 {
            self.set_waveform(i, 0).await?;
        }
        Ok(())
    }

    pub async fn play_waveform(&mut self, effect: u8) -> Result<(), Error<E>> {
        self.set_mode(Mode::InternalTrigger).await?;
        self.clear_waveform_sequence().await?;
        self.set_waveform(0, effect).await?;
        self.set_waveform(1, 0).await?;
        self.go().await
    }

    pub async fn set_rtp_input(&mut self, value: u8) -> Result<(), Error<E>> {
        self.write_register(registers::RTP_INPUT, value).await
    }

    pub async fn play_rtp(&mut self, value: u8) -> Result<(), Error<E>> {
        self.set_mode(Mode::RealTimePlayback).await?;
        self.set_rtp_input(value).await
    }

    async fn write_register(&mut self, reg: u8, value: u8) -> Result<(), Error<E>> {
        self.i2c
            .write(DRV2605L_ADDR, &[reg, value])
            .await
            .map_err(Error::I2c)
    }

    async fn read_register(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(DRV2605L_ADDR, &[reg], &mut buf)
            .await
            .map_err(Error::I2c)?;
        Ok(buf[0])
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Effect {
    StrongClick100 = 1,
    StrongClick60 = 2,
    StrongClick30 = 3,
    SharpClick100 = 4,
    SharpClick60 = 5,
    SharpClick30 = 6,
    SoftBump100 = 7,
    SoftBump60 = 8,
    SoftBump30 = 9,
    DoubleClick100 = 10,
    DoubleClick60 = 11,
    TripleClick100 = 12,
    SoftFuzz60 = 13,
    StrongBuzz100 = 14,
    Alert750ms = 15,
    Alert1000ms = 16,
    StrongClick1_100 = 17,
    StrongClick2_80 = 18,
    StrongClick3_60 = 19,
    StrongClick4_30 = 20,
    MediumClick1_100 = 21,
    MediumClick2_80 = 22,
    MediumClick3_60 = 23,
    SharpTick1_100 = 24,
    SharpTick2_80 = 25,
    SharpTick3_60 = 26,
    ShortDoubleClickStrong1_100 = 27,
    ShortDoubleClickStrong2_80 = 28,
    ShortDoubleClickStrong3_60 = 29,
    ShortDoubleClickStrong4_30 = 30,
    ShortDoubleClickMedium1_100 = 31,
    ShortDoubleClickMedium2_80 = 32,
    ShortDoubleClickMedium3_60 = 33,
    ShortDoubleSharpTick1_100 = 34,
    ShortDoubleSharpTick2_80 = 35,
    ShortDoubleSharpTick3_60 = 36,
    LongDoubleSharpClickStrong1_100 = 37,
    LongDoubleSharpClickStrong2_80 = 38,
    LongDoubleSharpClickStrong3_60 = 39,
    LongDoubleSharpClickStrong4_30 = 40,
    LongDoubleSharpClickMedium1_100 = 41,
    LongDoubleSharpClickMedium2_80 = 42,
    LongDoubleSharpClickMedium3_60 = 43,
    LongDoubleSharpTick1_100 = 44,
    LongDoubleSharpTick2_80 = 45,
    LongDoubleSharpTick3_60 = 46,
    Buzz1_100 = 47,
    Buzz2_80 = 48,
    Buzz3_60 = 49,
    Buzz4_40 = 50,
    Buzz5_20 = 51,
    PulsingStrong1_100 = 52,
    PulsingStrong2_60 = 53,
    PulsingMedium1_100 = 54,
    PulsingMedium2_60 = 55,
    PulsingSharp1_100 = 56,
    PulsingSharp2_60 = 57,
    TransitionClick1_100 = 58,
    TransitionClick2_80 = 59,
    TransitionClick3_60 = 60,
    TransitionClick4_40 = 61,
    TransitionClick5_20 = 62,
    TransitionClick6_10 = 63,
    TransitionHum1_100 = 64,
    TransitionHum2_80 = 65,
    TransitionHum3_60 = 66,
    TransitionHum4_40 = 67,
    TransitionHum5_20 = 68,
    TransitionHum6_10 = 69,
    TransitionRampDownLongSmooth1_100 = 70,
    TransitionRampDownLongSmooth2_100 = 71,
    TransitionRampDownMediumSmooth1_100 = 72,
    TransitionRampDownMediumSmooth2_100 = 73,
    TransitionRampDownShortSmooth1_100 = 74,
    TransitionRampDownShortSmooth2_100 = 75,
    TransitionRampDownLongSharp1_100 = 76,
    TransitionRampDownLongSharp2_100 = 77,
    TransitionRampDownMediumSharp1_100 = 78,
    TransitionRampDownMediumSharp2_100 = 79,
    TransitionRampDownShortSharp1_100 = 80,
    TransitionRampDownShortSharp2_100 = 81,
    TransitionRampUpLongSmooth1_100 = 82,
    TransitionRampUpLongSmooth2_100 = 83,
    TransitionRampUpMediumSmooth1_100 = 84,
    TransitionRampUpMediumSmooth2_100 = 85,
    TransitionRampUpShortSmooth1_100 = 86,
    TransitionRampUpShortSmooth2_100 = 87,
    TransitionRampUpLongSharp1_100 = 88,
    TransitionRampUpLongSharp2_100 = 89,
    TransitionRampUpMediumSharp1_100 = 90,
    TransitionRampUpMediumSharp2_100 = 91,
    TransitionRampUpShortSharp1_100 = 92,
    TransitionRampUpShortSharp2_100 = 93,
    TransitionRampDownLongSmooth1_50 = 94,
    TransitionRampDownLongSmooth2_50 = 95,
    TransitionRampDownMediumSmooth1_50 = 96,
    TransitionRampDownMediumSmooth2_50 = 97,
    TransitionRampDownShortSmooth1_50 = 98,
    TransitionRampDownShortSmooth2_50 = 99,
    TransitionRampDownLongSharp1_50 = 100,
    TransitionRampDownLongSharp2_50 = 101,
    TransitionRampDownMediumSharp1_50 = 102,
    TransitionRampDownMediumSharp2_50 = 103,
    TransitionRampDownShortSharp1_50 = 104,
    TransitionRampDownShortSharp2_50 = 105,
    TransitionRampUpLongSmooth1_50 = 106,
    TransitionRampUpLongSmooth2_50 = 107,
    TransitionRampUpMediumSmooth1_50 = 108,
    TransitionRampUpMediumSmooth2_50 = 109,
    TransitionRampUpShortSmooth1_50 = 110,
    TransitionRampUpShortSmooth2_50 = 111,
    TransitionRampUpLongSharp1_50 = 112,
    TransitionRampUpLongSharp2_50 = 113,
    TransitionRampUpMediumSharp1_50 = 114,
    TransitionRampUpMediumSharp2_50 = 115,
    TransitionRampUpShortSharp1_50 = 116,
    TransitionRampUpShortSharp2_50 = 117,
    LongBuzz100 = 118,
    SmoothHum1_50 = 119,
    SmoothHum2_40 = 120,
    SmoothHum3_30 = 121,
    SmoothHum4_20 = 122,
    SmoothHum5_10 = 123,
}

impl Effect {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub mod heartbeat {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "defmt", derive(Format))]
    pub struct HeartbeatPattern {
        pub bpm: u8,
        pub s1_amplitude: u8,
        pub s2_amplitude: u8,
    }

    impl Default for HeartbeatPattern {
        fn default() -> Self {
            Self {
                bpm: 70,
                s1_amplitude: 0x60,
                s2_amplitude: 0x38,
            }
        }
    }

    impl<I2C, E> Drv2605l<I2C>
    where
        I2C: I2c<Error = E>,
    {
        pub async fn play_heartbeat_builtin(&mut self) -> Result<(), Error<E>> {
            self.set_mode(Mode::InternalTrigger).await?;
            self.clear_waveform_sequence().await?;
            
            self.set_waveform(0, Effect::StrongClick100.as_u8()).await?;
            self.set_waveform(1, 0x81).await?;
            self.set_waveform(2, Effect::StrongClick60.as_u8()).await?;
            self.set_waveform(3, 0xB4).await?;
            self.set_waveform(4, 0).await?;
            
            self.go().await
        }

        pub async fn play_double_click_heartbeat(&mut self) -> Result<(), Error<E>> {
            self.play_waveform(Effect::DoubleClick100.as_u8()).await
        }

        pub async fn play_custom_heartbeat(
            &mut self,
            pattern: &HeartbeatPattern,
        ) -> Result<(), Error<E>> {
            self.set_mode(Mode::RealTimePlayback).await?;
            
            let cycle_ms = 60000 / pattern.bpm as u64;
            let systole_ms = (cycle_ms as f32 * 0.4) as u64;
            let diastole_ms = cycle_ms - systole_ms;
            
            loop {
                self.set_rtp_input(pattern.s1_amplitude).await?;
                Timer::after(Duration::from_millis(50)).await;
                self.set_rtp_input(pattern.s1_amplitude / 2).await?;
                Timer::after(Duration::from_millis(30)).await;
                self.set_rtp_input(0).await?;
                
                Timer::after(Duration::from_millis(systole_ms - 80)).await;
                
                self.set_rtp_input(pattern.s2_amplitude).await?;
                Timer::after(Duration::from_millis(40)).await;
                self.set_rtp_input(0).await?;
                
                Timer::after(Duration::from_millis(diastole_ms - 40)).await;
            }
        }
    }
}