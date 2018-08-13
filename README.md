# cRUST

cRUST vst software synthesizer plugin written in Rust using the vst crate.
It has 2 oscillators, each of which are switchable between sine, saw, square, and triangle waveforms.
cRUST also has a noise generator as well as an ADSR envelope filter. cRUST is a work in progress.

## How To Use
1. Clone or download the repository
2. Run `Cargo build --release` to build plugin
3. Run `./osx_vst_bundler.sh cRUST target/release/libcrust.dylib` to convert to .vst
4. Copy .vst into your plugins folder (see your DAW documentation)

## Status
### Done
* Switchable waveforms: sine, saw, square, triangle
* 2 independent oscillators
* Add noise generator
* Add ADSR envelope filter

### To Do
* Add polyphony
* Add velocity sensitivity
* Add cutoff/resonance
* Add lfo

## Useful Links
https://crates.io/crates/vst

https://github.com/rust-dsp/rust-vst

## License
This program is licensed under the "MIT License". Please see the file `COPYING` in the source distribution of this software for license terms.

## Contact
randyrollofson@gmail.com

http://www.randyrollofson.com
