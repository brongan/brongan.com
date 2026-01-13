use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioContextState, GainNode, OscillatorNode};

#[derive(Clone)]
pub struct Beeper {
    ctx: AudioContext,
    _oscillator: OscillatorNode,
    gain: GainNode,
}

impl Beeper {
    pub fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;

        let oscillator = ctx.create_oscillator()?;
        oscillator.set_type(web_sys::OscillatorType::Square); // Chip-8 style square wave
        oscillator.frequency().set_value(440.0); // A4 pitch

        let gain = ctx.create_gain()?;
        gain.gain().set_value(0.0); // Start muted

        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&ctx.destination())?;

        oscillator.start()?;

        Ok(Self {
            ctx,
            _oscillator: oscillator,
            gain,
        })
    }

    pub fn play(&self) {
        self.gain.gain().set_value(0.1);
    }

    pub fn pause(&self) {
        self.gain.gain().set_value(0.0);
    }

    pub fn resume_context(&self) {
        if self.ctx.state() == AudioContextState::Suspended {
            let _ = self.ctx.resume();
        }
    }
}
