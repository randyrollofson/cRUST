#[macro_use]
extern crate vst;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Plugin, Info};
use vst::event::Event;
use vst::api::Events;
use std::f64::consts::PI;


struct Crust {
    time: f64,
    midi_note: u8,
    //velocity: u8,
    sample_rate: f64,
}

impl Default for Crust {
    fn default() -> Crust {
        Crust {
            time: 0.0,
            midi_note: 0,
            //velocity: 0,
            sample_rate: 48000.0,
        }
    }
}

fn midi_note_num_to_freq(midi_note_number: u8) -> f64 {
    ((midi_note_number as f64 - 69.0) / 12.0).exp2() * 440.0
}

impl Crust {
    fn process_midi(&mut self, midi_data: [u8; 3]) {
        match midi_data[0] {
            128 => self.note_off(midi_data[1]),
            144 => self.note_on(midi_data[1]),
            _ => (),
        }
    }

    fn note_on(&mut self, note: u8) {
        self.midi_note = note;
    }

    fn note_off(&mut self, note: u8) {
        if self.midi_note == note {
            self.midi_note = 0;
        }
    }
}

impl Plugin for Crust {
    fn get_info(&self) -> Info {
        Info {
            name: "Crust".to_string(),
            unique_id: 736251,
            inputs: 2,
            outputs: 2,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi(ev.data),
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let waveform = "square";
        let samples = buffer.samples();
        let sample = (1.0 / self.sample_rate) as f64;

        for (input_buffer, output_buffer) in buffer.zip() {
            let mut time = self.time;

            for (_, output_sample) in input_buffer.iter().zip(output_buffer) {
                if self.midi_note != 0 {
                    let osc1;
                    if waveform == "square" {
                        osc1 = (time * midi_note_num_to_freq(self.midi_note) * 2.0 * PI).sin();
                    } else if waveform == "sawtooth" {
                        osc1 = (time *  midi_note_num_to_freq(self.midi_note)) - ((time *  midi_note_num_to_freq(self.midi_note)).floor()) - 0.5;
                    } else if waveform == "triangle" {
                        osc1 = (((time *  midi_note_num_to_freq(self.midi_note)) - ((time *  midi_note_num_to_freq(self.midi_note)).floor()) - 0.5).abs() - 0.25) * 4.0;
                    } else if waveform == "square" {
                        if (time * midi_note_num_to_freq(self.midi_note) * 2.0 * PI).sin() >= 0.0 {
                            osc1 = 1.0;
                        } else {
                            osc1 = -1.0;
                        }
                    } else {
                        osc1 = 0.0;
                    }
                    *output_sample = osc1 as f32;
                    time += sample;
                } else {
                    *output_sample = 0.0;
                }
            }
        }
        self.time += samples as f64 * sample;
    }
}

plugin_main!(Crust);
