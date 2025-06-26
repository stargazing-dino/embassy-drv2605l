# embassy-drv2605l

An async driver for the DRV2605L haptic motor controller, designed for use with Embassy-rs on embedded systems.

## Features

- Async I2C communication using `embedded-hal-async`
- Support for both ERM (Eccentric Rotating Mass) and LRA (Linear Resonant Actuator) motors
- Access to all 123 built-in haptic effects
- Real-time playback (RTP) mode for custom patterns
- Waveform sequencing (up to 8 effects in sequence)
- Pre-built heartbeat patterns with physiologically accurate timing
- Optional `defmt` support for debugging

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
embassy-drv2605l = "0.1.0"
```

## Basic Usage

### Simple Effect Playback

```rust
use embassy_drv2605l::{Drv2605l, Effect};

// Create the driver with your I2C instance
let mut haptic = Drv2605l::new(i2c);

// Initialize the driver (defaults to LRA motor)
haptic.init().await?;

// Play a strong click effect
haptic.play_waveform(Effect::StrongClick100.as_u8()).await?;

// Play a double click
haptic.play_waveform(Effect::DoubleClick100.as_u8()).await?;
```

### Motor Type Configuration

```rust
use embassy_drv2605l::{Drv2605l, MotorType};

let mut haptic = Drv2605l::new(i2c);

// Configure for ERM motor before initialization
haptic.set_motor_type(MotorType::ERM).await?;
haptic.init().await?;
```

### Waveform Sequences

Create complex haptic patterns by chaining multiple effects:

```rust
// Clear any existing sequence
haptic.clear_waveform_sequence().await?;

// Build a sequence: strong click, pause, double click
haptic.set_waveform(0, Effect::StrongClick100.as_u8()).await?;
haptic.set_waveform(1, 0x81).await?;  // 10ms pause
haptic.set_waveform(2, Effect::DoubleClick60.as_u8()).await?;
haptic.set_waveform(3, 0).await?;  // End sequence

// Play the sequence
haptic.go().await?;
```

### Real-Time Playback (RTP)

For custom haptic patterns with precise control:

```rust
use embassy_drv2605l::Mode;
use embassy_time::{Duration, Timer};

// Switch to RTP mode
haptic.set_mode(Mode::RealTimePlayback).await?;

// Create a simple pulse pattern
for _ in 0..3 {
    haptic.set_rtp_input(0x7F).await?;  // 50% intensity
    Timer::after(Duration::from_millis(50)).await;
    
    haptic.set_rtp_input(0x00).await?;  // Off
    Timer::after(Duration::from_millis(100)).await;
}
```

## Heartbeat Patterns

The driver includes specialized support for creating realistic heartbeat haptic patterns:

### Using Built-in Effects

```rust
use embassy_drv2605l::heartbeat;

// Simple heartbeat using library effects
haptic.play_double_click_heartbeat().await?;

// More complex heartbeat sequence
haptic.play_heartbeat_builtin().await?;
```

### Custom Heartbeat Patterns

```rust
use embassy_drv2605l::heartbeat::{HeartbeatPattern};

// Create a custom heartbeat pattern
let pattern = HeartbeatPattern {
    bpm: 70,           // 70 beats per minute
    s1_amplitude: 0x60,  // "Lub" strength (75%)
    s2_amplitude: 0x38,  // "Dub" strength (45%)
};

// This will play continuously
haptic.play_custom_heartbeat(&pattern).await?;
```

## Available Effects

The library includes all 123 built-in DRV2605L effects. Some commonly used ones:

- `StrongClick100` - Strong, sharp click at full intensity
- `SoftBump100` - Soft, rounded bump sensation
- `DoubleClick100` - Two quick clicks (great for heartbeats)
- `Alert750ms` - Long alert vibration
- `TransitionRampUpShortSmooth1_100` - Smooth ramp up effect

See the `Effect` enum in the source code for the complete list.

## Pause/Delay Encoding

When building waveform sequences, you can insert delays between effects:

- Delays are encoded as `0x80 | delay_value`
- Each unit represents 10ms
- Maximum delay: 1270ms (0xFF)

Example:
```rust
haptic.set_waveform(1, 0x81).await?;  // 10ms delay
haptic.set_waveform(2, 0x8A).await?;  // 100ms delay
haptic.set_waveform(3, 0xFF).await?;  // 1270ms delay
```

## Power Management

```rust
// Enter standby mode to save power
haptic.enter_standby().await?;

// Wake up from standby
haptic.exit_standby().await?;
```

## Example: Complete Haptic Feedback System

```rust
use embassy_drv2605l::{Drv2605l, Effect, Mode};
use embassy_time::{Duration, Timer};

async fn haptic_demo(i2c: impl I2c) -> Result<(), Error> {
    let mut haptic = Drv2605l::new(i2c);
    haptic.init().await?;
    
    // Button press feedback
    haptic.play_waveform(Effect::SharpClick100.as_u8()).await?;
    Timer::after(Duration::from_millis(500)).await;
    
    // Success notification
    haptic.play_waveform(Effect::DoubleClick100.as_u8()).await?;
    Timer::after(Duration::from_millis(500)).await;
    
    // Error notification
    for _ in 0..3 {
        haptic.play_waveform(Effect::StrongBuzz100.as_u8()).await?;
        Timer::after(Duration::from_millis(200)).await;
    }
    
    // Enter low-power mode
    haptic.enter_standby().await?;
    
    Ok(())
}
```

## Hardware Setup

The DRV2605L communicates over I2C at address `0x5A`. When using the Adafruit breakout board:

- Connect VIN to 3.3V (not 5V)
- Connect GND to ground
- Connect SDA to your I2C data line
- Connect SCL to your I2C clock line
- The EN pin must be high for operation (pulled up on the breakout)

## Motor Selection

**LRA (Linear Resonant Actuator)** - Recommended for:
- Precise, crisp haptic feedback
- Lower power consumption
- Heartbeat and subtle patterns

**ERM (Eccentric Rotating Mass)** - Better for:
- Strong vibrations
- Legacy compatibility
- Simple on/off patterns

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.