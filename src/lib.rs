// Copyright © 2018 Randy Rollofson
//     ALL RIGHTS RESERVED
//     [This program is licensed under the "MIT License"]
//     Please see the file COPYING in the source
//     distribution of this software for license terms.

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
    dist: f32,
}

impl Default for Crust {
    fn default() -> Crust {
        Crust {
            time: 0.0,
            midi_note: 0,
            sample_rate: 48000.0,
            volume: 0.0,
            wave_index: 0.0,
            dist: 0.0,
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
fn distortion(input: f32, dist:f32) -> f32 {
    let gain = 5.0;
    let q = input / input.abs();
    let y = q * (1.0 - (gain * (q * input)).exp());
    let z = dist * y + (1.0 - dist) * input;

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

fn build_sound(input: f32, dist: f32) -> f32 {
    let sound: f32 = distortion(input, dist);
    let final_sound: f32 = overdrive(sound);

    final_sound
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
            2 => self.dist,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, val: f32) {
        println!("{:?}", val);
        match index {
            0 => self.wave_index = val,
            1 => self.volume = val,
            2 => self.dist = val,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "waveform".to_string(),
            1 => "volume".to_string(),
            2 => "distortion".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", ((self.wave_index) * 4.0).round()),
            1 => format!("{}%", ((self.volume) * 100.0).round()),
            2 => format!("{}%", ((self.dist) * 100.0).round()),
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

                    *output_sample = build_sound(wave, self.dist);

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

#[test]
fn test_sine_wave() {
    assert_eq!(sine_wave(0, 0.0, 0.0), 0.0);
    assert_eq!(sine_wave(69, 1.0, 0.0005989), 1.0);
    assert_eq!(sine_wave(69, 1.0, 0.0017045), -1.0);
    // assert_eq!(sine_wave(60, 0.75, 2400.0), 0.5876097);
    // assert_eq!(sine_wave(80, 0.75, 2400.0), -0.22450724);
    // assert_eq!(sine_wave(100, 0.75, 2400.0), 0.41266116);
    // assert_eq!(sine_wave(127, 0.75, 2400.0), 0.078091696);
}

// #[test]
// fn test_sawtooth_wave() {
//     assert_eq!(sawtooth_wave(0, 0.75, 2400.0), 0.31304818);
//     assert_eq!(sawtooth_wave(20, 0.75, 2400.0), 0.15347774);
//     assert_eq!(sawtooth_wave(60, 0.75, 2400.0), -0.10745893);
//     assert_eq!(sawtooth_wave(80, 0.75, 2400.0), 0.0362878);
//     assert_eq!(sawtooth_wave(100, 0.75, 2400.0), -0.30545467);
//     assert_eq!(sawtooth_wave(127, 0.75, 2400.0), -0.0124512445);
// }
//
// #[test]
// fn test_square_wave() {
//     assert_eq!(square_wave(0, 0.75, 2400.0), -0.3);
//     assert_eq!(square_wave(20, 0.75, 2400.0), -0.3);
//     assert_eq!(square_wave(60, 0.75, 2400.0), 0.3);
//     assert_eq!(square_wave(80, 0.75, 2400.0), -0.3);
//     assert_eq!(square_wave(100, 0.75, 2400.0), 0.3);
//     assert_eq!(square_wave(127, 0.75, 2400.0), 0.3);
// }
//
// #[test]
// fn test_triangle_wave() {
//     assert_eq!(triangle_wave(0, 0.75, 2400.0), -0.3);
//     assert_eq!(triangle_wave(20, 0.75, 2400.0), -0.3);
//     assert_eq!(triangle_wave(60, 0.75, 2400.0), 0.3);
//     assert_eq!(triangle_wave(80, 0.75, 2400.0), -0.3);
//     assert_eq!(triangle_wave(100, 0.75, 2400.0), 0.3);
//     assert_eq!(triangle_wave(127, 0.75, 2400.0), 0.3);
// }

#[test]
fn test_midi_note_num_to_freq() {
    assert_eq!(midi_note_num_to_freq(0).round() , 8.0);
    assert_eq!(midi_note_num_to_freq(10).round() , 15.0);
    assert_eq!(midi_note_num_to_freq(20).round() , 26.0);
    assert_eq!(midi_note_num_to_freq(30).round() , 46.0);
    assert_eq!(midi_note_num_to_freq(40).round() , 82.0);
    assert_eq!(midi_note_num_to_freq(50).round() , 147.0);
    assert_eq!(midi_note_num_to_freq(60).round() , 262.0);
    assert_eq!(midi_note_num_to_freq(69).round() , 440.0);
    assert_eq!(midi_note_num_to_freq(70).round() , 466.0);
    assert_eq!(midi_note_num_to_freq(80).round() , 831.0);
    assert_eq!(midi_note_num_to_freq(90).round() , 1480.0);
    assert_eq!(midi_note_num_to_freq(100).round() , 2637.0);
    assert_eq!(midi_note_num_to_freq(110).round() , 4699.0);
    assert_eq!(midi_note_num_to_freq(120).round() , 8372.0);
    assert_eq!(midi_note_num_to_freq(127).round() , 12544.0);
}
