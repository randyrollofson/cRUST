// Copyright © 2018 Randy Rollofson
//     ALL RIGHTS RESERVED
//     [This program is licensed under the "MIT License"]
//     Please see the file COPYING in the source
//     distribution of this software for license terms.

#[macro_use]
extern crate vst;
extern crate rand;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Plugin, Info};
use vst::event::Event;
use vst::api::Events;
use std::f64::consts::PI;
use rand::random;

struct Oscillator {
    midi_note: u8,
    volume: f32,
    wave_index: f32,
    detune: f32,
}

impl Default for Oscillator {
    fn default() -> Oscillator {
        Oscillator {
            midi_note: 0,
            volume: 0.5,
            wave_index: 0.0,
            detune: 0.0,
        }
    }
}

struct Crust {
    time: f64,
    midi_note: u8,
    sample_rate: f64,
    oscillators: Vec<Oscillator>,
    noise: f32,
    attack: f32,
    release: f32,
    start_time: f64,
    end_time: f64,
    note_on: bool,
    master_vol: f32,
}

impl Default for Crust {
    fn default() -> Crust {
        Crust {
            time: 0.0,
            midi_note: 0,
            sample_rate: 44100.0,
            oscillators: vec![Default::default(), Default::default()],
            noise: 0.0,
            attack: 0.05,
            release: 0.7,
            start_time: 0.0,
            end_time: 0.0,
            note_on: false,
            master_vol: 1.0,
        }
    }
}

fn sine_wave(midi_note: u8, volume: f32, time: f64, detune: f32) -> f32 {
    volume * (time as f32 * midi_note_num_to_freq(midi_note, detune) as f32 * 2.0 * PI as f32).sin()
}

fn sawtooth_wave(midi_note: u8, volume: f32, time: f64, detune: f32) -> f32 {
    volume * (time *  midi_note_num_to_freq(midi_note, detune) - ((time *  midi_note_num_to_freq(midi_note, detune)).floor()) - 0.5) as f32
}

fn square_wave(midi_note: u8, volume: f32, time: f64, detune: f32) -> f32 {
    if (time * midi_note_num_to_freq(midi_note, detune) * 2.0 * PI).sin() as f32 >= 0.0 {
        volume * 0.4 // not using 1.0 in order to balance with other waveforms
    } else {
        volume * -0.4
    }
}

fn triangle_wave(midi_note: u8, volume: f32, time: f64, detune: f32) -> f32 {
    volume * ((((time *  midi_note_num_to_freq(midi_note, detune)) - ((time *  midi_note_num_to_freq(midi_note, detune)).floor()) - 0.5).abs() - 0.25) * 4.0) as f32
}

fn midi_note_num_to_freq(midi_note_number: u8, detune: f32) -> f64 {
    (((midi_note_number as f64 - 69.0) / 12.0).exp2() * 440.0) - detune as f64
}

fn generate_attack(attack: f32, time: f64, volume: f32) -> f32 {
    if time <= attack as f64 {
        (time as f32 / attack) * volume
    } else {
        volume
    }
}

fn generate_release(release: f32, end_time: f64, volume: f32) -> f32 {
    if end_time <= release as f64 {
        (end_time as f32 / release) * (0.0 - volume) + volume
    } else {
        0.0
    }
    // ((time - end_time) as f32 / release) * (0.0 - volume) + volume
}

fn lpf(input: f32) -> f32 {
    (((2.0 * PI as f32 * 500.0) / input).tan() - 1.0) / (((2.0 * PI as f32 * 500.0) / input).tan() + 1.0)
}

// Distortion formula based on
// https://ccrma.stanford.edu/~orchi/Documents/DAFx.pdf
fn distortion(input: f32, dist: f32, dist_volume: f32) -> f32 {
    let gain = 5.0;
    let q = input / input.abs();
    let y = q * (1.0 - (gain * (q * input)).exp());
    let z = dist * y + (1.0 - dist) * input;

    dist_volume * z
    // if dist_volume == 0.0 {
    //     input
    // } else {
    //     dist_volume * ((input * (1.0 - (dist * (input).exp2() / input.abs()).exp())) / input.abs())
    // }
}

// Overdrive formula based on
// https://ccrma.stanford.edu/~orchi/Documents/DAFx.pdf
fn overdrive(input: f32) -> f32 {
    if input == 0.0 {
        input
    } else {
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
}

fn noise(dist: f32) -> f32 {
    dist * (((0.02 * (random::<f32>() * 2.0 - 1.0)) / 1.02) * 3.5)
}

impl Crust {
    fn process_midi_data(&mut self, midi_data: [u8; 3]) {
        match midi_data[0] {
            128 => self.note_off(),
            144 => self.note_on(midi_data[1]),
            // 224 => self.pitch_bend(midi_data[1]),
            _ => (),
        }
    }

    fn note_on(&mut self, note: u8) {
        self.midi_note = note;
        self.oscillators[0].midi_note = note;
        self.oscillators[1].midi_note = note;
        self.note_on = true;
        self.start_time = 0.0;
    }

    fn note_off(&mut self) {
        self.note_on = false;
        self.end_time = 0.0;
    }
}

impl Plugin for Crust {
    fn get_info(&self) -> Info {
        Info {
            name: "Crust".to_string(),
            unique_id: 736251,
            inputs: 2,
            outputs: 2,
            parameters: 10,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.oscillators[0].wave_index,
            1 => self.oscillators[0].volume,
            2 => self.oscillators[0].detune,
            3 => self.oscillators[1].wave_index,
            4 => self.oscillators[1].volume,
            5 => self.oscillators[1].detune,
            6 => self.noise,
            7 => self.attack,
            8 => self.release,
            9 => self.master_vol,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, val: f32) {
        match index {
            0 => self.oscillators[0].wave_index = val,
            1 => self.oscillators[0].volume = val,
            2 => self.oscillators[0].detune = val * 10.0,
            3 => self.oscillators[1].wave_index = val,
            4 => self.oscillators[1].volume = val,
            5 => self.oscillators[1].detune = val * 10.0,
            6 => self.noise = val,
            7 => self.attack = val * 5.0,
            8 => self.release = val * 5.0,
            9 => self.master_vol = val,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Osc 1 waveform".to_string(),
            1 => "Osc 1 volume".to_string(),
            2 => "Osc 1 detune".to_string(),
            3 => "Osc 2 waveform".to_string(),
            4 => "Osc 2 volume".to_string(),
            5 => "Osc 2 detune".to_string(),
            6 => "Noise".to_string(),
            7 => "Attack".to_string(),
            8 => "Release".to_string(),
            9 => "Master volume".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", (self.oscillators[0].wave_index * 3.0).round()),
            1 => format!("{}%", (self.oscillators[0].volume * 100.0).round()),
            2 => format!("{}", self.oscillators[0].detune),
            3 => format!("{}", (self.oscillators[0].wave_index * 3.0).round()),
            4 => format!("{}%", (self.oscillators[1].volume * 100.0).round()),
            5 => format!("{}", self.oscillators[1].detune),
            6 => format!("{}%", (self.noise * 100.0).round()),
            7 => format!("{}", self.attack),
            8 => format!("{}", self.release),
            9 => format!("{}%", (self.master_vol* 100.0).round()),
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

            for (_, output_sample) in input_buffer.iter().zip(output_buffer) {
                let mut wave1;
                let mut wave2;

                let mut osc1_volume = self.oscillators[0].volume;
                let mut osc2_volume = self.oscillators[1].volume;

                if self.oscillators[0].wave_index >= 0.0 && self.oscillators[0].wave_index < 0.33 {
                    wave1 = sine_wave(self.oscillators[0].midi_note, osc1_volume, time, self.oscillators[0].detune);
                } else if self.oscillators[0].wave_index >= 0.33 && self.oscillators[0].wave_index < 0.66 {
                    wave1 = sawtooth_wave(self.oscillators[0].midi_note, osc1_volume, time, self.oscillators[0].detune);
                } else if self.oscillators[0].wave_index >= 0.66 && self.oscillators[0].wave_index < 1.0 {
                    wave1 = square_wave(self.oscillators[0].midi_note, osc1_volume, time, self.oscillators[0].detune);
                } else if self.oscillators[0].wave_index >= 1.0 {
                     wave1 = triangle_wave(self.oscillators[0].midi_note, osc1_volume, time, self.oscillators[0].detune);
                } else {
                     wave1 = 0.0;
                }

                if self.oscillators[1].wave_index >= 0.0 && self.oscillators[1].wave_index < 0.33 {
                    wave2 = sine_wave(self.oscillators[1].midi_note, osc2_volume, time, self.oscillators[1].detune);
                } else if self.oscillators[1].wave_index >= 0.33 && self.oscillators[1].wave_index < 0.66 {
                    wave2 = sawtooth_wave(self.oscillators[1].midi_note, osc2_volume, time, self.oscillators[1].detune);
                } else if self.oscillators[1].wave_index >= 0.66 && self.oscillators[1].wave_index < 1.0 {
                    wave2 = square_wave(self.oscillators[1].midi_note, osc2_volume, time, self.oscillators[1].detune);
                } else if self.oscillators[1].wave_index >= 1.0 {
                     wave2 = triangle_wave(self.oscillators[1].midi_note, osc2_volume, time, self.oscillators[1].detune);
                } else {
                     wave2 = 0.0;
                }

                if self.note_on == true {
                    let mut attack_volume = generate_attack(self.attack, self.start_time, self.master_vol);
                    *output_sample = attack_volume as f32 * (wave1 + wave2) + noise(self.noise);

                    self.start_time += sample;
                } else {
                    let mut release_volume = generate_release(self.release, self.end_time, self.master_vol);

                    if release_volume < 0.0001 {
                        *output_sample = 0.0;
                    } else {
                        *output_sample = release_volume * (wave1 + wave2) + noise(self.noise);
                    }

                    self.end_time += sample;
                }
                time += sample;
            }
        }
        
        self.time += samples as f64 * sample;
    }
}

plugin_main!(Crust);

#[test]
fn test_sine_wave() {
    assert_eq!(sine_wave(0, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(sine_wave(69, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(sine_wave(69, 1.0, 0.0005682, 0.0), 1.0);
    assert_eq!(sine_wave(69, 1.0, 0.0017045, 0.0), -1.0);
}

#[test]
fn test_sawtooth_wave() {
    assert_eq!(sawtooth_wave(0, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(sawtooth_wave(69, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(sawtooth_wave(69, 1.0, 0.00454545454, 0.0), 0.5);
    assert_eq!(sawtooth_wave(69, 1.0, 0.00454545455, 0.0), -0.5);
}

#[test]
fn test_square_wave() {
    assert_eq!(square_wave(0, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(square_wave(69, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(square_wave(69, 1.0, 0.0005682, 0.0), 0.4);
    assert_eq!(square_wave(69, 1.0, 0.0017045, 0.0), -0.4);
}

#[test]
fn test_triangle_wave() {
    assert_eq!(triangle_wave(0, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(triangle_wave(69, 0.0, 0.0, 0.0), 0.0);
    assert_eq!(sine_wave(69, 1.0, 0.0005682, 0.0), 1.0);
    assert_eq!(sine_wave(69, 1.0, 0.0017045, 0.0), -1.0);
}

#[test]
fn test_midi_note_num_to_freq() {
    assert_eq!(midi_note_num_to_freq(21, 0.0), 27.5);
    assert_eq!(midi_note_num_to_freq(33, 0.0), 55.0);
    assert_eq!(midi_note_num_to_freq(45, 0.0), 110.0);
    assert_eq!(midi_note_num_to_freq(57, 0.0), 220.0);
    assert_eq!(midi_note_num_to_freq(69, 0.0), 440.0);
    assert_eq!(midi_note_num_to_freq(81, 0.0), 880.0);
    assert_eq!(midi_note_num_to_freq(93, 0.0), 1760.0);
    assert_eq!(midi_note_num_to_freq(105, 0.0), 3520.0);
}

#[test]
fn test_distortion() {
    assert_eq!(distortion(0.75, 0.0, 1.0), 0.75);
    assert_eq!(distortion(0.75, 0.32, 1.0), -12.776747);
    assert_eq!(distortion(0.75, 0.50, 1.0), -20.385542);
    assert_eq!(distortion(0.75, 0.75, 1.0), -30.953312);
    assert_eq!(distortion(0.75, 1.0, 1.0), -41.521084);
}

#[test]
fn test_overdrive() {
    assert_eq!(overdrive(0.0), 0.0);
    assert_eq!(overdrive(0.32), 0.64);
    assert_eq!(overdrive(0.50), 0.5285955);
    assert_eq!(overdrive(0.75), 1.0);
    assert_eq!(overdrive(1.0), 1.0);
}
