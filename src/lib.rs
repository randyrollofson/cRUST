#[macro_use]
extern crate vst;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Plugin, Info};
use vst::event::Event;
use vst::api::{Events};
use std::f64::consts::PI;

#[derive(Default)]
struct Crust {
    notes: u8
}

impl Plugin for Crust {
    fn get_info(&self) -> Info {
        Info {
            name: "Crust".to_string(),
            unique_id: 736251,
            inputs: 0,
            outputs: 2,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        144 => self.notes += 1u8,
                        128 => self.notes -= 1u8,
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let sine_wave: f64 = (440.0 * 2.0 * PI).sin();
        if self.notes == 0 {
            return
        }

        let (_, output_buffer) = buffer.split();
        for output_channel in output_buffer.into_iter() {
            for output_sample in output_channel {
                *output_sample = sine_wave as f32;
            }
        }
    }
}

plugin_main!(Crust);
