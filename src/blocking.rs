use crate::common::{DRV2605L_ADDR, Error, Library, Mode, MotorType};
use crate::registers;
use embedded_hal::i2c::I2c;

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

    pub fn init(&mut self) -> Result<(), Error<E>> {
        self.reset()?;
        
        // Wait 2ms after reset
        // In blocking mode, user must handle delay externally
        
        self.exit_standby()?;
        
        if self.motor_type == MotorType::LRA {
            self.write_register(registers::FEEDBACK_CONTROL, 0x80)?;
            self.set_library(Library::LRA)?;
        } else {
            self.write_register(registers::FEEDBACK_CONTROL, 0x00)?;
            self.set_library(Library::LibraryB)?;
        }
        
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::MODE, 0x80)
    }

    pub fn exit_standby(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::MODE, 0x00)
    }

    pub fn enter_standby(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::MODE, 0x40)
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Error<E>> {
        let current = self.read_register(registers::MODE)?;
        let new_value = (current & 0xF8) | (mode as u8);
        self.write_register(registers::MODE, new_value)
    }

    pub fn set_library(&mut self, library: Library) -> Result<(), Error<E>> {
        self.write_register(registers::LIBRARY_SELECTION, library as u8)
    }

    pub fn set_motor_type(&mut self, motor_type: MotorType) -> Result<(), Error<E>> {
        self.motor_type = motor_type;
        
        match motor_type {
            MotorType::LRA => {
                self.write_register(registers::FEEDBACK_CONTROL, 0x80)?;
                self.set_library(Library::LRA)
            }
            MotorType::ERM => {
                self.write_register(registers::FEEDBACK_CONTROL, 0x00)?;
                self.set_library(Library::LibraryB)
            }
        }
    }

    pub fn go(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::GO, 0x01)
    }

    pub fn stop(&mut self) -> Result<(), Error<E>> {
        self.write_register(registers::GO, 0x00)
    }

    pub fn is_playing(&mut self) -> Result<bool, Error<E>> {
        let go_reg = self.read_register(registers::GO)?;
        Ok(go_reg & 0x01 != 0)
    }

    pub fn set_waveform(&mut self, slot: u8, effect: u8) -> Result<(), Error<E>> {
        if slot > 7 {
            return Err(Error::InvalidParameter);
        }
        
        let reg = registers::WAVEFORM_SEQUENCER_1 + slot;
        self.write_register(reg, effect)
    }

    pub fn clear_waveform_sequence(&mut self) -> Result<(), Error<E>> {
        for i in 0..8 {
            self.set_waveform(i, 0)?;
        }
        Ok(())
    }

    pub fn play_waveform(&mut self, effect: u8) -> Result<(), Error<E>> {
        self.set_mode(Mode::InternalTrigger)?;
        self.clear_waveform_sequence()?;
        self.set_waveform(0, effect)?;
        self.set_waveform(1, 0)?;
        self.go()
    }

    pub fn set_rtp_input(&mut self, value: u8) -> Result<(), Error<E>> {
        self.write_register(registers::RTP_INPUT, value)
    }

    pub fn play_rtp(&mut self, value: u8) -> Result<(), Error<E>> {
        self.set_mode(Mode::RealTimePlayback)?;
        self.set_rtp_input(value)
    }

    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), Error<E>> {
        self.i2c
            .write(DRV2605L_ADDR, &[reg, value])
            .map_err(Error::I2c)
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(DRV2605L_ADDR, &[reg], &mut buf)
            .map_err(Error::I2c)?;
        Ok(buf[0])
    }
    
    pub fn set_rated_voltage(&mut self, mv: u16) -> Result<(), Error<E>> {
        let value = ((mv as u32 * 255) / 5600) as u8;
        self.write_register(registers::RATED_VOLTAGE, value)
    }
    
    pub fn set_overdrive_voltage(&mut self, mv: u16) -> Result<(), Error<E>> {
        let value = ((mv as u32 * 255) / 5600) as u8;
        self.write_register(registers::OVERDRIVE_CLAMP_VOLTAGE, value)
    }
    
    pub fn get_device_id(&mut self) -> Result<u8, Error<E>> {
        let status = self.read_register(registers::STATUS)?;
        Ok((status >> 5) & 0x07)
    }
}