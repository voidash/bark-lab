# Bark Lab video renderer

An animated four-dog band, driven by the rendered audio's per-voice envelope and spectrum.
The dog voices are a **sample-based performance**, separate from the experimental physical
vocal model in the main app. All character art is original SVG/React source.

The renderer takes a local audio master and JSON animation data. The generic preparation
script uses the bundled CC0 bark/howl PCM, independent pitch/time processing and an optional
local backing track. It never downloads a song or reads credentials. Source-separated
vocals, if supplied, are used for pitch analysis only. No human lead waveform is mixed
into the dog voice bus. Imperfect source separation can leave traces in a backing stem.

```sh
cd video
bun install --frozen-lockfile
uv venv .audio-env --python 3.12
uv pip install --python .audio-env/bin/python numpy==2.5.4 scipy==1.18.1 librosa==0.11.0 soundfile==0.13.1
.audio-env/bin/python prepare_audio.py --score examples/warmup.json --outdir .runtime --duration 9 --lead-offset 1
bun run typecheck
bun run render output/warmup.mp4 --props .runtime/props.json --public-dir .runtime --codec h264
```

For a local cover, supply your own score JSON (`notes` with finite `t`, `duration`, `pitch`,
`amplitude`), `--backing` and optionally `--vocals`, plus `--title`, `--duration` and
`--lead-offset`. An optional `--deep-bark` accepts another source sample. Input paths and
formats are validated; the preparation manifest records their hashes. Relative pitch
intervals are preserved, but irregular bark samples do not promise a precise fundamental.

The commissioned Who Let the Dogs Out excerpt, its score, original song/stems and rendered
cover are local review artifacts outside Git. They are not bundled with this public repo.
The included warm-up is an original conventional arpeggio. No rights to third-party music
are conferred by the project's MIT license.

`src/` contains the frame-driven Remotion composition; `prepare_audio.py` prepares audio and
animation data; `examples/warmup.json` is the source-only reproducible example. Creator:
Codex GPT-6, session /root, 2026-11-09 local date. Git history and source documentation
record authorship. Retain as active project source. Generated media, environments and
caches are disposable after their consumer finishes; document their inventories at the
chosen output directory before retaining them. CC0 sample provenance is under
`../crates/chimera/src/creatures/data/README.md`.

`preview.png` is the 1920×1080 frame-120 poster rendered from this original vector
composition on 2026-11-08 UTC by Codex GPT-6, session /root, on
cdjks-MacBook-Pro-2.local, originally at
`/Users/cdjk/github/dog/research/dog-video/preview-layout.png`. Size: 323,530 bytes.
It contains no audio; the local review page retains a verified identical copy.
SHA-256: `da6c8c4618574ab776fb364aa31db4906454035de5072d17e051d9c05f843810`.
Status: reference; owner cdjk; current consumer: the repository README preview.
Last reviewed 2026-11-08 UTC; next review 2026-12-08. Retain while used by the README;
replace when the artwork changes. Restore from Git, or render frame 120 with Remotion
using local prepared props. This project's MIT license covers the original artwork.
