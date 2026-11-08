# Dog voice source data

Created by Codex GPT-6, session /root, 2026-11-08T13:08:52.752728+00:00, on cdjks-MacBook-Pro-2.local. Owner cdjk. Original path /Users/cdjk/github/dog/chimera-vocal-synth/crates/chimera/src/creatures/data. Status active: embedded source material for the recognizable dog voice, replacing the user-rejected physical model. Last review 2026-11-08; next review 2026-11-09. Retain while the dog voice uses these files; delete only after its source references and tests are removed or replaced. NOT on HF.

## Licence and provenance

Both recordings are released under **CC0 1.0** by their original uploaders. Licence confirmed in the actual Freesound source HTML before download. These public-domain-dedicated assets are the dog exception to CHIMERA’s originally sample-free architecture. No GPL synthesis code was copied. This is recorded-source playback/resynthesis, not a claim of physical canine synthesis.

## Inventory and reproduction

- README.md: this self-documenting source provenance and inventory.
- manifest.json: exact source URLs, authors (in source-page paths), source checksums, crop intervals, format, normalization and derived checksums.
- dog-bark.pcm: Single Dog Bark by kwahmah_02, Freesound 277058, CC0; whole public preview, normalized and faded for playback.
- dog-howl.pcm: dog howling.wav by simcotter, Freesound 115357, CC0; 18.55–20.75 s of the public preview, normalized and faded.

Format: mono, 24,000 Hz, signed PCM16 little endian with no header. Original public previews were decoded with ffmpeg to mono float32 at 24 kHz. Crop to the manifest sample interval, normalize peak to .85, apply symmetric linear fades of the stated sample count, quantize with round(sample*32767). No training. No claim that either recording represents all dogs or a specific calibrated breed.

Current local source-preview copy: /Users/cdjk/github/dog/research/dog-v1, documented there. The final PCM contains the retained portions only; it is distinct from the MP3 source preview. Restore by downloading the exact manifest preview URLs, verifying source SHA256, applying this conversion and checking the output hashes.

- dog-bark.pcm: 18672 bytes; SHA256 f0751650440fbb230f60962cadedd6baf4a1c05f342bfb5e257b2286dfc74250; 9336 frames. Original source https://freesound.org/people/kwahmah_02/sounds/277058/. Current consumer: creatures::recorded_dog; usage CC0 1.0.
- dog-howl.pcm: 105600 bytes; SHA256 a4495b01514021f4dac3e3a41b04419b917dd669f30a129c9b7923d049cd5f75; 52800 frames. Original source https://freesound.org/people/simcotter/sounds/115357/. Current consumer: creatures::recorded_dog; usage CC0 1.0.
