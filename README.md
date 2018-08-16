# cRUST

cRUST is a vst software synthesizer plugin written in Rust using the vst crate.
It has 2 oscillators, each of which are switchable between sine, saw, square, and triangle waveforms.
cRUST also has a noise generator as well as an ADSR envelope filter. cRUST is a work in progress and
has only been fully tested on macOS High Sierra using Cubase and Ableton DAWs.

## How To Use
1. Clone or download the repository
2. Run `Cargo build --release` to build plugin
3. Mac: run `./osx_vst_bundler.sh cRUST target/release/libcrust.dylib` to create a vst bundle and import into your DAW
   Linux: navigate to `cRUST/target/release/` and copy `libcrust.so` and import into your DAW
4. Copy .vst into your plugins folder (see your DAW documentation)

## Status
### Done
* Switchable waveforms: sine, saw, square, triangle
* 2 oscillators
* Add noise generator
* Add ADSR envelope filter
* Add polyphony

### To Do
* Fix envelope release
* Add velocity sensitivity
* Add cutoff/resonance
* Add lfo
* Get distortion and overdrive to work properly

## Useful Links
https://crates.io/crates/vst

https://github.com/rust-dsp/rust-vst

## License
This program is licensed under the "MIT License". Please see the file `COPYING` in the source distribution of this software for license terms.

## Contact
randyrollofson@gmail.com

http://www.randyrollofson.com
