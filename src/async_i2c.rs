use crate::common::{DRV2605L_ADDR, Error, Library, Mode, MotorType};
use crate::registers;
use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

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
    
    pub async fn set_rated_voltage(&mut self, mv: u16) -> Result<(), Error<E>> {
        let value = ((mv as u32 * 255) / 5600) as u8;
        self.write_register(registers::RATED_VOLTAGE, value).await
    }
    
    pub async fn set_overdrive_voltage(&mut self, mv: u16) -> Result<(), Error<E>> {
        let value = ((mv as u32 * 255) / 5600) as u8;
        self.write_register(registers::OVERDRIVE_CLAMP_VOLTAGE, value).await
    }
    
    pub async fn get_device_id(&mut self) -> Result<u8, Error<E>> {
        let status = self.read_register(registers::STATUS).await?;
        Ok((status >> 5) & 0x07)
    }
    
    pub async fn auto_calibrate(&mut self) -> Result<(), Error<E>> {
        self.set_mode(Mode::AutoCalibration).await?;
        self.go().await?;
        
        // Wait for calibration to complete
        let mut timeout = 100;
        while self.is_playing().await? && timeout > 0 {
            Timer::after(Duration::from_millis(10)).await;
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(Error::CalibrationFailed);
        }
        
        // Check if calibration was successful
        let status = self.read_register(registers::STATUS).await?;
        if status & 0x08 != 0 {
            return Err(Error::CalibrationFailed);
        }
        
        Ok(())
    }
}