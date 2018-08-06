# cRUST

cRUST is a work in progress vst software synthesizer written in Rust using the vst crate.
It uses a single oscillator and is switchable between sine, saw, square, and triangle waveforms.

## How To Use
1. Clone or download the repository
2. Run `Cargo build --release` to build plugin.
3. Run `./osx_vst_bundler.sh cRUST target/release/libcrust.dylib` to convert to .vst.
4. Copy .vst into your plugins folder (see your DAW documentation).

## Status
### Done
* Switchable waveforms: sine, saw, square, triangle
* Added distortion DSP
* Added overdrive DSP

### To Do
* Add velocity sensitivity
* Add cutoff/resonance
* Add envelope filter
* Add polyphony

## Useful Links
https://crates.io/crates/vst
https://github.com/rust-dsp/rust-vst

## License
This program is licensed under the "MIT License". Please see the file `COPYING` in the source distribution of this software for license terms.

## Contact
randyrollofson@gmail.com

http://www.randyrollofson.com
