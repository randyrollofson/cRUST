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
    sample_rate: f64,
    volume: f32,
    wave_index: f32,
    distortion: f32,
    overdrive: f32,
}

impl Default for Crust {
    fn default() -> Crust {
        Crust {
            time: 0.0,
            midi_note: 0,
            sample_rate: 48000.0,
            volume: 0.0,
            wave_index: 0.0,
            distortion: 0.0,
            overdrive: 0.0,
        }
    }
}

fn sine_wave(midi_note: u8, volume: f32, time: f64) -> f32 {
    volume as f32 * (time * midi_note_num_to_freq(midi_note) * 2.0 * PI).sin() as f32
}

fn sawtooth_wave(midi_note: u8, volume: f32, time: f64) -> f32 {
    volume * (time *  (midi_note_num_to_freq(midi_note)) - ((time *  midi_note_num_to_freq(midi_note)).floor()) - 0.5) as f32
}

fn square_wave(midi_note: u8, volume: f32, time: f64) -> f32 {
    if (time * midi_note_num_to_freq(midi_note) * 2.0 * PI).sin() as f32 >= 0.0 {
        volume * 0.4 // not using 1.0 in order to balance with other waveforms
    } else {
        volume * -0.4
    }
}

fn triangle_wave(midi_note: u8, volume: f32, time: f64) -> f32 {
    volume * ((((time *  midi_note_num_to_freq(midi_note)) - ((time *  midi_note_num_to_freq(midi_note)).floor()) - 0.5).abs() - 0.25) * 4.0) as f32
}

fn midi_note_num_to_freq(midi_note_number: u8) -> f64 {
    ((midi_note_number as f64 - 69.0) / 12.0).exp2() * 440.0
}


// Distortion formula based on
// https://ccrma.stanford.edu/~orchi/Documents/DAFx.pdf
fn distortion(input: f32, distortion:f32) -> f32 {
    let gain = 5.0;
    let q = input / input.abs();
    let y = q * (1.0 - (gain * (q * input)).exp());
    let z = distortion * y + (1.0 - distortion) * input;
    z
}

// Overdrive formula based on
// https://ccrma.stanford.edu/~orchi/Documents/DAFx.pdf
fn overdrive(input: f32) -> f32 {
    let output: f32;
    if input < 0.33 {
        output = 2.0 * input;
    } else if input >= 0.33 && input < 0.66 {
        output = (3.0 - (2.0 - (3.0 * input)).exp2()) / 3.0;
    } else if input >= 0.66 && input <= 1.0 {
        output = 1.0;
    } else {
        output = input;
    }

    output
}

impl Crust {
    fn process_midi_data(&mut self, midi_data: [u8; 3]) {
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
            parameters: 4,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.wave_index,
            1 => self.volume,
            2 => self.distortion,
            3 => self.overdrive,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, val: f32) {
        println!("{:?}", val);
        match index {
            0 => self.wave_index = val,
            1 => self.volume = val,
            2 => self.distortion = val,
            3 => self.overdrive = val,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "waveform".to_string(),
            1 => "volume".to_string(),
            2 => "distortion".to_string(),
            3 => "overdrive".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", ((self.wave_index) * 4.0).round()),
            1 => format!("{}%", ((self.volume) * 100.0).round()),
            2 => format!("{}", ((self.distortion) * 10.0).round()),
            2 => format!("{}", ((self.overdrive) * 10.0).round()),
            _ => "".to_string(),
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_data(ev.data),
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let sample = (1.0 / self.sample_rate) as f64;

        for (input_buffer, output_buffer) in buffer.zip() {
            let mut time = self.time;
            let mut volume = self.volume;

            for (_, output_sample) in input_buffer.iter().zip(output_buffer) {

                if self.midi_note != 0 {
                    let wave;
                    //let dist;
                    if self.wave_index >= 0.0 && self.wave_index < 0.25 {
                        wave = sine_wave(self.midi_note, volume, time);
                    } else if self.wave_index >= 0.25 && self.wave_index < 0.5 {
                        wave = sawtooth_wave(self.midi_note, volume, time);
                    } else if self.wave_index >= 0.5 && self.wave_index < 0.75 {
                        wave = square_wave(self.midi_note, volume, time);
                    } else if self.wave_index >= 0.75 && self.wave_index <= 1.0 {
                         wave = triangle_wave(self.midi_note, volume, time);
                    } else {
                         wave = 0.0;
                    }

                    //*output_sample = distortion(wave, self.crust);
                    //*output_sample = wave;

                    *output_sample = overdrive(wave);

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
