# Dog voice listening prototype

Created by Codex (GPT-6), session `/root`, 2026-11-08 UTC, in
`/Users/cdjk/github/dog/chimera-vocal-synth`. Owner: cdjk. Status: active source
documentation; review after the first listening comparison. Git history records revisions.
The physical model bundles no recordings. The default Dog voice now embeds two CC0
recordings; see `crates/chimera/src/creatures/data/README.md` for authors, URLs and checksums.

## Hypothesis and limits

A parameter-controlled mammalian source/filter can support short bark-like attacks and
tonal howls without changing the human voice engine. Start with that hypothesis, and judge
dog-likeness before expanding the interface. Pitch accuracy and numerical stability do not
establish perceptual realism.

The design separates breath, phonation and mouth articulation, as in
[Moore 2016](https://www.isca-archive.org/interspeech_2016/moore16_interspeech.pdf).
The initial 17 cm pharyngeal/oral tract is a design hypothesis near the paper's scaling for
a medium mammal. The tube proportions, areas and envelopes are engineering choices,
not measurements of a specific breed. There is no claim of anatomically calibrated canine
phonation. [Soundgen](https://doi.org/10.3758/s13428-018-1095-7) informs the separation
of pitch, noise and spectral controls; its implementation was not copied.

`dog.rs` uses the existing branched mammal airway and parametric glottis. Bark calls
combine a fast pressure envelope, falling pitch, a closing mouth and rough phonation.
Howls use a narrower mouth and low roughness. The yodel alternates pitch and timbre;
it is not a physiological register-transition model.

Two fixed call scores play C C G G A A G at 120 BPM. Calls run on the existing creature
audio clock. This is a listening demonstration, not a general song editor. Keyboard
transposition follows the existing creature pitch limits; extreme transposition can clamp
individual call pitches and change intervals.

## Reproduce

```sh
bun install --frozen-lockfile
bun run test
bun run typecheck
cargo run --release -p chimera-cli -- render creature dog out/dog --split
bun run dev
```

The app opens on Bestiary with Dog selected; starting audio selects the same dog engine.
The left screen identifies the dog vocal tract and hides human body controls.
The CLI's `--split` option creates independent clips and a
JSON manifest; each clip begins from rest. The manifest contains its actual sample rate,
file names, durations, peak and full-clip RMS, rather than relying on hard-coded offsets.
Generated output requires a provenance inventory at its destination (see standing retention
policy); no downloaded reference should be committed to this repository.

## Validation

`tests/dog.rs` checks the retained `dog-model` physical prototype: pitch on clean held howls at 44.1, 48 and 96 kHz, melody intervals,
audibility, release, finite airway pressure at parameter extremes, and identical output
across render block sizes. The existing generic tests include the dog in allocation checks,
call-key resolution and the browser worklet telemetry test. The old TypeScript fixtures
continue to assert the original creatures exactly; their historical registry is independent
of newly added Rust creatures.

The listening comparison should use independently recorded references, randomized playback
order and matched levels. Rate canine character, sound quality and tune recognition
separately. A sample transposed by playback rate also moves its formants and duration;
that makes a simple baseline, not a claim of an optimal sample instrument. A small reference
set is enough to expose an obviously poor prototype, not to establish general realism.

## First listening result

On 2026-11-08 the user rejected the synthetic bark as not sounding like a bark. The
perceptual gate therefore failed despite passing the numerical tests. The current timbre
is a rejected baseline; further bark-source work must compare actual recorded barks before
being described as an improvement. Starting on a human head was a separate startup bug;
the default view now uses the dog tract and audio engine, with a browser regression check.

## Recorded voice revision (2026-11-08)

The user again rejected the bark. Numerical pitch tests cannot repair that perceptual
failure. The `dog` reference plays a clean dog bark by kwahmah_02 and a cropped husky howl
by simcotter (both CC0, Freesound). This is a sample instrument, not a successful physical
canine model. `dog-model` preserves the physical baseline without presenting it as realistic.

The Howl call plays the natural recording through its ending. Holding a key loops a steady
portion with an 80 ms crossfade; Yodel makes octave jumps, and the two tune calls transpose
the sources through Twinkle's opening phrase. Playback-rate transposition also changes
formants and duration; the pitch centre is nominal and natural intonation remains.

The dog's anatomy stays schematic. Only the recorded source amplitude and audio spectrum
animate; simulated airway pressures remain zero and the pressure toggle is hidden.
`tests/recorded_dog.rs` checks source waveform correlation at two pitches, natural bark
completion, six-second howl sustain, release, block independence and honest telemetry.

Render `dog-model` instead of `dog` in the CLI to reproduce the rejected physical baseline.
Do not describe the new version as perceptually approved until the user has listened.

The subsequent pressure-driven source is documented in [dog-physics.md](dog-physics.md).
It is now the default; the recorded voice is explicitly named Dog reference.
