# embassy-drv2605l

Driver for the DRV2605L haptic motor controller with both blocking and async I2C support.

## Features

- **Dual mode**: Both blocking and async I2C implementations
- **Motor support**: ERM and LRA haptic motors
- **123 built-in effects**: From clicks to buzzes to complex transitions
- **Custom patterns**: Real-time playback mode for precise control
- **Waveform sequencing**: Chain up to 8 effects
- **Heartbeat patterns**: Realistic physiological haptic feedback
- **`no_std` compatible**: For embedded systems
- **Optional `defmt`**: Debug support when needed

## Installation

```toml
[dependencies]
# For async (default)
embassy-drv2605l = "0.1.0"

# For blocking only
embassy-drv2605l = { version = "0.1.0", default-features = false, features = ["blocking"] }

# For both blocking and async
embassy-drv2605l = { version = "0.1.0", features = ["blocking", "async"] }
```

## Quick Start

### Async Mode

```rust
use embassy_drv2605l::{Drv2605l, Effect};

let mut haptic = Drv2605l::new(i2c);
haptic.init().await?;

// Play a click
haptic.play_waveform(Effect::StrongClick100.as_u8()).await?;
```

### Blocking Mode

```rust
use embassy_drv2605l::{Drv2605l, Effect};

let mut haptic = Drv2605l::new(i2c);
haptic.init()?;

// Play a click  
haptic.play_waveform(Effect::StrongClick100.as_u8())?;
```

## Common Usage Patterns

### Motor Configuration

```rust
use embassy_drv2605l::{Drv2605l, MotorType};

let mut haptic = Drv2605l::new(i2c);
haptic.set_motor_type(MotorType::ERM).await?; // Default is LRA
haptic.init().await?;
```

### Waveform Sequences

```rust
// Chain multiple effects
haptic.clear_waveform_sequence().await?;
haptic.set_waveform(0, Effect::StrongClick100.as_u8()).await?;
haptic.set_waveform(1, 0x81).await?;  // 10ms pause  
haptic.set_waveform(2, Effect::DoubleClick60.as_u8()).await?;
haptic.set_waveform(3, 0).await?;  // End marker
haptic.go().await?;
```

### Real-Time Playback

```rust
// Custom intensity control
haptic.set_mode(Mode::RealTimePlayback).await?;
haptic.set_rtp_input(0x7F).await?;  // 50% intensity
haptic.set_rtp_input(0xFF).await?;  // Full intensity
haptic.set_rtp_input(0x00).await?;  // Stop
```

### Heartbeat Patterns

```rust
use embassy_drv2605l::HeartbeatPattern;

// Built-in heartbeat
haptic.play_heartbeat_builtin().await?;

// Custom heartbeat (async only)
let pattern = HeartbeatPattern {
    bpm: 70,
    s1_amplitude: 0x60,  // "Lub" strength  
    s2_amplitude: 0x38,  // "Dub" strength
};
haptic.play_custom_heartbeat(&pattern).await?;
```

## Popular Effects

- **Clicks**: `StrongClick100`, `SharpClick60`, `SoftBump100`
- **Alerts**: `DoubleClick100`, `TripleClick100`, `Alert750ms`  
- **Transitions**: `TransitionRampUpShortSmooth1_100`
- **Buzzes**: `StrongBuzz100`, `SmoothHum1_50`

123 effects total - see `Effect` enum for complete list.

## Timing in Sequences

Insert delays between effects: `0x80 | (delay_ms / 10)`

```rust
haptic.set_waveform(1, 0x81).await?;  // 10ms
haptic.set_waveform(2, 0x8A).await?;  // 100ms  
haptic.set_waveform(3, 0xFF).await?;  // 1270ms (max)
```

## Hardware

**I2C Connection** (Address: `0x5A`)
- VIN → 3.3V
- GND → Ground  
- SDA → I2C Data
- SCL → I2C Clock
- EN → High (or leave - has pullup)

**Motor Types**
- **LRA**: Precise feedback, low power, best for subtle patterns
- **ERM**: Strong vibration, simple on/off control

## License

MIT OR Apache-2.0