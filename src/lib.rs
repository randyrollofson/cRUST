#[macro_use]
extern crate vst;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Plugin, Info};
use vst::event::Event;
use vst::api::Events;
use std::f64::consts::PI;

// #[derive(Default)]
struct Crust {
    time: f64,
    midi_note: u8,
    //velocity: u8,
    sample_rate: f64,
    volume: f64,
    // sine: f32,
    // saw: f32,
    // square: f32,
    // triangle: f32,
    //waveforms: Vec<String>,
    //waveform: f64,
}

impl Default for Crust {
    fn default() -> Crust {
        Crust {
            time: 0.0,
            midi_note: 0,
            //velocity: 0,
            sample_rate: 48000.0,
            //waveforms: vec!["sine".to_string(), "sawtooth".to_string(), "square".to_string(), "triangle".to_string()],
            //waveform: sine_wave(),
            volume: 0.5,
        }
    }
}

fn sine_wave(midi_note: u8, volume: f64, time: f64) -> f32 {
    volume as f32 * (time * midi_note_num_to_freq(midi_note) * 2.0 * PI).sin() as f32
}

fn sawtooth_wave(midi_note: u8, volume: f64, time: f64) -> f32 {
    (time *  (midi_note_num_to_freq(midi_note)) - ((time *  midi_note_num_to_freq(midi_note)).floor()) - 0.5) as f32
}

fn square_wave(midi_note: u8, volume: f64, time: f64) -> f32 {
    if (time * midi_note_num_to_freq(midi_note) * 2.0 * PI).sin() >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

fn triangle_wave(midi_note: u8, volume: f64, time: f64) -> f32 {
    ((((time *  midi_note_num_to_freq(midi_note)) - ((time *  midi_note_num_to_freq(midi_note)).floor()) - 0.5).abs() - 0.25) * 4.0) as f32
}

// fn get_waveform(midi_note, time: f64)

fn midi_note_num_to_freq(midi_note_number: u8) -> f64 {
    ((midi_note_number as f64 - 69.0) / 12.0).exp2() * 440.0
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
            parameters: 1,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.volume as f32,
            // 1 => self.saw,
            // 2 => self.square,
            // 3 => self.triangle,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, val: f32) {
        match index {
            0 => self.volume = val as f64,
            // 1 => self.saw = val,
            // 2 => self.square = val,
            // 3 => self.triangle = val,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "volume".to_string(),
            // 1 => "saw".to_string(),
            // 2 => "square".to_string(),
            // 3 => "triangle".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}%", (self.volume) * 100.0),
            // 1 => "Saw",
            // 2 => "Square",
            // 3 => "Triangle",
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
        //self.sample_rate = 48000.0;
        //self.time = 0.0;
        //let waveform = "sine";
        let samples = buffer.samples();
        let sample = (1.0 / self.sample_rate) as f64;

        for (input_buffer, output_buffer) in buffer.zip() {
            let mut time = self.time;
            // self.sine = sine_wave(self.midi_note, time);
            // self.saw = sawtooth_wave(self.midi_note, time);
            // self.square = square_wave(self.midi_note, time);
            // self.triangle = triangle_wave(self.midi_note, time);

            for (_, output_sample) in input_buffer.iter().zip(output_buffer) {
                // self.sine = sine_wave(self.midi_note, time);
                if self.midi_note != 0 {
                    *output_sample = sine_wave(self.midi_note, self.volume, time);
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
