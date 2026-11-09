# Pressure-driven dog experiment

Codex GPT-6, session /root, 2026-11-08 UTC. Owner cdjk. Active source documentation;
review after listening feedback. Original workspace /Users/cdjk/github/dog/chimera-vocal-synth.
Review interface: http://127.0.0.1:58071/dog-physics/. Native outputs are the Rust source
and independent WAVs, inventoried in the research directory's README.

`dog-physics` is the default. Four tissue masses (lower/upper edges of left/right folds)
obey mass–spring–damper equations with cubic restoring forces and collision. Quasi-steady
jet separation determines aerodynamic forces. RK4 advances the tissue; its minimum
aperture drives the acoustic network's glottal port. Instantaneous tracheal/pharyngeal
pressures feed back to the folds. No recorded excitation, prescribed pulse, phase
accumulator or logistic chaos map supplies this source's waveform.

Starting equations/constants: [Neubauer 2004, §2.5.1](https://edoc.hu-berlin.de/bitstreams/b1b3ad61-9eef-444a-8dd7-b9d0e2b39719/download)
(Steinecke–Herzel); nonlinear restoring forces are motivated by
[Assaneo & Trevisan 2013](https://arxiv.org/html/1312.2950). Generic mammalian constants
and an initial canine-like airway are not measurements of a breed or an anatomical
inversion from sound. This reduced model is not a full fluid/structure continuum solver.

The user found the first mechanical voice too clean and identified useful grunge in v0.
V0's irregularity comes from its programmed pulse perturbations. A bounded mechanical
sweep tested detuning, throat entrance, pressure and damping. At nominal 330 Hz,
28 cmH2O and damping multiplier .7, median YIN aperiodicity was .0013 (regular folds,
wide throat), .0005 (regular, narrow), .5167 (detuned, wide), and .4978 (detuned, narrow).
Detuned tissue modes with lower damping produced the irregularity; narrowing alone did
not. These finite windows do not establish deterministic chaos or canine realism.

The Bark candidate uses effective detuning .85, damping .7, throat entrance .4 cm²
and 28 cmH2O at nominal 330 Hz. The earlier seven-note phrase retains its notes, 120 BPM,
gate and mouth gestures. Actual breath pressure is explicitly coordinated with pitch
to approximately preserve P/Q as tension changes. The fixed acoustic tract breaks exact
scaling, and arbitrary keyboard transposition does not re-fit the pressure schedule.

The same-arrangement diagnostic remains more regular on upper notes than v0: median
short-frame residual approximately .111 (candidate), .060 (clean physical source),
.400 (v0), .345 (recording). This is not a perceptual grunge rating: glides/transients
affect it. The candidate is not a complete match to v0's consistent texture.

The user's musical target is https://www.youtube.com/watch?v=ojULkWEUsPs, Baha Men's
Who Let the Dogs Out. A local first-hook arrangement uses 23 estimated gestures from
12.66–27.66 s. Offline ASR estimates event timing; YIN estimates pitch from a mixed track.
No lyric transcript or original song audio is in the output. Pitch tracking can follow
instruments or make octave errors; it is not a verified transcription. The local score
and rendered cover remain outside Git; no redistribution rights to the original are claimed.

`tests/dog_physics.rs` guards pressure-driven motion/decay, tension response, acoustic
feedback, explicit failures, stable parameter extremes, release, block independence
and a reproducible irregular regime. Generic tests guard zero allocation. Source faults
latch, stop output and surface in telemetry/UI; the oscillator is not silently reset.
Historical human/animal fixtures remain unchanged. Dog reference (`dog`) and v0
(`dog-model`) stay available for comparison.

The lab's dog view now includes a dedicated source inspector. It exposes the solved
left/right lower/upper tissue displacements, glottal aperture, glottal volume flow,
lung boundary pressure, and sub/supraglottal pressures. Tube lanes show the solver's
configured lengths and cross-sections. The head is an original reference-informed
sagittal cutaway, with no centimetre ruler or segmented scan. The four-mass diagram magnifies displacement and
shows live snapshots; it is not a reconstructed slow-motion cycle or MRI anatomy.
Reading telemetry is tested not to change the rendered samples, and the aperture is
checked against the four solved displacements. The existing named-source telemetry
layout carries these scalars; its worklet round trip remains covered by the ABI test.

This corrects an interface gap relative to upstream CHIMERA's human Play view, which
has separate tract, fold stroboscope, flow and spectral displays. The Bestiary path had
only a schematic source ring and tube lanes. Human 3D anatomy and its phase-locked
stroboscope remain inappropriate substitutes for canine geometry and irregular cycles.
The dog inspector links canine tract measurements, bark HNR research, measured canine
fluid–structure work and Moore's real-time mammalian synthesizer, alongside the model's
generic tissue constants and unvalidated canine identity.

Reproduce with `cargo run --release -p chimera-cli -- render creature dog-physics
../research/dog-physics --split`, examples `probe_dog_regimes` (optionally `--focus`)
and `render_dog_score` (score JSON, output WAV), then `bun tools/dog-physics-analysis.ts
../research/dog-physics` and `python3 ../research/dog-physics/make-review.py`.
The website compares matched-level WAVs. Numerical checks do not replace user listening.

The controllable cutaway shows tongue, palate/velum, epiglottis, jaw, larynx and
oral/nasal spaces, informed by the University of Minnesota canine sagittal anatomy
reference (https://vanat.ahc.umn.edu/carnLabs/Lab23/Img23-11.html). No atlas image
is copied into the repository. Jaw opening changes the outlet profile; tongue
elevation (0–1) applies up to 75% local area reduction, positioned along the oral
tube at normalized 0.25–0.80. Tongue-root retraction (0–1) reduces the pharyngeal
profile by up to 55%. Nasal opening controls the velum port. Larynx descent adds
0–2 cm to pharyngeal length, preserving the fixed acoustic grid's CFL bound.
These ranges and mappings are engineering assumptions, not measured canine muscle
kinematics. New articulators at their defaults preserve the prior sound.

Dragging, arrow keys and labeled sliders send the same real engine parameters.
Manual edits cancel call automation; Sustain supplies breath for continuous
listening. Reset anatomy restores resting geometry without changing tissue
settings. The oral lumen follows solver cross-sections through an assumed fixed
out-of-plane depth. Other tissue contours and jaw rotation are illustrative.
The epiglottis and nasal turbinates are landmarks, not independently simulated
structures. The four-mass physical source and its live inspector remain separate.
