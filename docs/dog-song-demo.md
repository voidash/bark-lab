# Same-lab dog song comparison

Created 2026-11-09 UTC by Codex GPT-6 session /root, owner cdjk. Active source
documentation for the requested comparison/video; review after owner listening.
Source workspace /Users/cdjk/github/dog/chimera-vocal-synth, baseline f4b8941.
Original generic player/capture code is MIT. No music score or cover audio is in Git.

Open the actual lab with `?tissue=canine&song=dog-song`. A local score at
`public/shows/dog-song/score.json` supplies a short hook sketch. That directory is
ignored because the repository excludes copyrighted arrangements from Git.
Without a local score the UI displays its loading error instead of substituting
another sound. The generic player works with original/local scores of the same schema.

The two takes use the same nominal note controls, event times and breath gates.
**Dog reference** plays the existing CC0 bark/howl PCM through its transposing
sample player, with neutral brightness and its existing onset/loop behaviour.
Its drawing is illustrative; its airway pressure telemetry remains zero.
**Dog physics** uses the six-mass body–cover source and the acoustic airway,
including actual pitch locking and timbre limitations. Its pressure, aperture,
flow and body/cover positions come from the simulation. Neither recording nor
pitch correction supplies the physical voice's excitation.

The existing mixed-track ASR/YIN arrangement is estimated, not a verified melody
transcription. The local sketch uses its first nine events twice, with a howl
lead and bark response. No lyric transcript or original artist audio is included.
Measured held-note probes show that the physical source often locks near a tract
resonance over a range of nominal pitches. This demo therefore demonstrates a
musical control attempt and a sound-quality gap, not accurate physical singing.
The reference retains natural pitch contours; nominal Hz is not an exact F0 claim.

## Player

`CreatureScore` is `{keys: [{t, values, curve?}, ...]}`. Times are sorted seconds
relative to start; values name the selected creature's parameters, `gate`, or
registered `!events`; curve is `lin` (default), `exp`, or `step`. Rust rejects
unknown names, non-finite/out-of-range values, missing zero-time start, missing
final zero gate, more than 2000 keys, or time beyond 120 seconds. JSON loading
is capped at 1 MiB. Validation completes before replacing the current call.

Loading/compilation allocates outside rendering. The existing call evaluator
runs the compiled plan on the creature's own audio clock. No JavaScript timer
schedules musical notes. Stopping hands controls back to the performer, as with
normal calls; the UI releases held breath before starting a song. Switching
creature cancels the old voice/score. Telemetry identifies a local performance as
`score`, via an extra call ID in the switch metadata; binary record sizes do not
change. Invalid user scores report a recoverable score error; Wasm traps still
latch an engine failure.

Tests cover sample equality across block sizes, end/stop release, parameter
ownership, invalid-input preservation, event/parameter validation, real source
identity through Wasm/worklet and allocation-free render/cancel/preset playback
after loading. Loading retains the preset call buffer's capacity.

## Video capture

Run `bun tools/capture-dog-demo.ts URL OUTPUT_DIRECTORY`. It operates the actual
production lab at 1920×1440, records Chromium viewport frames with timestamps,
and records that same page's post-master WebAudio stream with MediaRecorder.
Each take asserts source identity, audible output, no physical fault and score
completion. Reference takes must have zero simulated airway pressure.

FFmpeg uses the captured frame intervals and audio/video epoch offset to encode
each take, then concatenates reference first and physics second. It introduces
no alternate synth render, artist vocal, backing track or staged anatomy movie.
Audio and video are verified after encoding; output method labels are part of
the real lab. Intermediates expire after QA; final clips, compact validation and
provenance are inventoried in `../research/dog-song/README.md` for the owner's
active review (next Nov 10, expiry Nov 16 unless renewed). Exact discarded
historical frame captures are not restorable; the source and local score recreate
a new live take. No new training model or HF repository is involved.
