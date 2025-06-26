#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg(feature = "blocking")]
pub mod blocking {
    use super::*;
    use crate::blocking::Drv2605l;
    use crate::common::{Effect, Error, Mode};
    use embedded_hal::i2c::I2c;

    impl<I2C, E> Drv2605l<I2C>
    where
        I2C: I2c<Error = E>,
    {
        pub fn play_heartbeat_builtin(&mut self) -> Result<(), Error<E>> {
            self.set_mode(Mode::InternalTrigger)?;
            self.clear_waveform_sequence()?;
            
            self.set_waveform(0, Effect::StrongClick100.as_u8())?;
            self.set_waveform(1, 0x81)?; // Wait with bit 7 set
            self.set_waveform(2, Effect::StrongClick60.as_u8())?;
            self.set_waveform(3, 0xB4)?; // Wait 340ms (180 * 2ms)
            self.set_waveform(4, 0)?;
            
            self.go()
        }

        pub fn play_double_click_heartbeat(&mut self) -> Result<(), Error<E>> {
            self.play_waveform(Effect::DoubleClick100.as_u8())
        }
        
        pub fn start_custom_heartbeat(&mut self, pattern: &HeartbeatPattern) -> Result<(), Error<E>> {
            self.set_mode(Mode::RealTimePlayback)?;
            self.set_rtp_input(pattern.s1_amplitude)
        }
    }
}

#[cfg(feature = "async")]
pub mod async_i2c {
    use super::*;
    use crate::async_i2c::Drv2605l;
    use crate::common::{Effect, Error, Mode};
    use embassy_time::{Duration, Timer};
    use embedded_hal_async::i2c::I2c;

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