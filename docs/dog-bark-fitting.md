# Physical bark calibration

The target is a recognizable isolated bark from the simulated larynx and airway.
Recordings are evaluation references, never excitation or residual signals mixed into
the physical render. No human vocal waveform or musical accompaniment is used.

The previous candidate used one hand-picked 17 cm tract and adjusted fold detuning for
irregularity. That did not establish a convincing bark. This experiment exposes independent
oral tract length, oral cavity cross-section, glottal rest gap and flow-noise controls,
then fits those along with breath, tension, detuning, damping and jaw/pressure timing.
The mechanics remain pressure-driven bilateral two-mass folds with collision, coupled
to the branched FDTD airway. This is a reduced model, not a subject-specific MRI model.

[Riede & Fitch (1999)](https://pubmed.ncbi.nlm.nih.gov/10504322/) measured canine tract
lengths and growl formants; the revised chapter's Table 5.2 in
[Riede's dissertation](https://edoc.hu-berlin.de/server/api/core/bitstreams/b829fee3-e313-47ab-a312-06f4b137c684/content)
reports lengths 6.9–22.4 cm. Their non-uniform formant spacing argues against assuming a
uniform tube. These growl measurements are anatomical constraints, not bark-specific
area functions. The present length search uses only 17–22.4 cm: shortening the existing
fixed grid would violate its CFL design. A smaller dog requires rebuilding that grid.
Cross-sectional and gap bounds are engineering search limits, not measured dog anatomy.

[Riede et al.'s bark HNR study](https://pubmed.ncbi.nlm.nih.gov/11681395/)
and [Tokuda (2018)](https://www.jstage.jst.go.jp/article/ase/126/1/126_171130/_html/-char/en)
motivate keeping periodic tissue motion, irregularity and flow noise distinct. More
roughness is not by itself more canine. The source equations/constants are audited
against [Neubauer (2004), §2.5.1](https://edoc.hu-berlin.de/bitstreams/b1b3ad61-9eef-444a-8dd7-b9d0e2b39719/download);
their reference parameters describe human male phonation, so the present source is
still a generic mammalian approximation requiring canine calibration. No additional
membrane is assumed from a report about a single dog–wolf hybrid.

```sh
cargo run --release -p chimera --example fit_dog_bark -- ../research/dog-rework
cargo run --release -p chimera --example fit_dog_bark -- --render ../research/dog-rework/candidate.json ../research/dog-rework/replay
```

The fitter uses the bundled CC0 bark by kwahmah_02. It searches 15 bounded controls
using 64 deterministic initial points and three bounded Nelder–Mead runs. The objective
combines 10 ms amplitude envelopes, coarse spectral energy in three time regions
at two resolutions, and four short-frame YIN aperiodicity estimates. A first spectral-only
pass selected a source more periodic than the recording and was rejected before this
second pass. Periodicity is now a calibration feature; it is no longer an independent
validation target. Numerical failures receive a penalty and are counted by reason;
they are not silently reset. Only final configurations, audio and compact metrics are
saved. The `--render` path takes parameter JSON alone and does not read reference audio.

`reference.wav`, `before.wav`, `candidate.wav`, `candidate.json` and `fit.json` are
local outputs. `before.wav` uses the same new pressure/jaw envelope renderer at the
previous source settings; the historical app bark is a separate comparison. WAVs have
equal peak amplitude, which is not equal perceived loudness. No single scalar objective
is a dog-likeness score. A one-recording fit can overfit the reference and cannot uniquely
recover anatomy; independent recordings, a wider morphology range and listener judgment
remain necessary. Keep this candidate separate from the app's default until listening.

The first measured pass reduced coarse spectral RMS error from 11.3231 to 7.7417 dB,
but its separate short-frame residual median was .0873 versus .2581 in the recording:
it became too periodic. The second pass trades some spectral match (8.1390 dB) for a
residual median of .1700. Envelope RMS error is .2191 versus .1890 before fitting, so
the envelope did not improve. These results identify remaining errors; they do not
establish a better-sounding bark. Both passes completed without mechanical faults.
The second pass used 1,688 evaluations and a 21.72 cm oral tract; nominal pitch hit
the lower search boundary near 150 Hz. Boundary contact and the imperfect envelope
are reasons to question this model family/control mapping rather than endlessly tune
the same objective. Do not promote this result to a validated dog voice.

An independent frequency-domain test checks that increasing tract length lowers its
resonances without changing source settings. Existing causality, fault, stable-extreme,
allocation and block-independence checks remain in place. Original human/animal fixtures
are not deliberately changed.

## Timing-only trial, 2026-11-09

```sh
cargo run --release -p chimera --example fit_dog_bark -- --pulse ../research/dog-rework/candidate.json ../research/dog-pulse
cargo run --release -p chimera --example fit_dog_bark -- --score ../research/dog-pulse/candidate.json
```

The previous candidate starts late and has excessive duration: 10 ms RMS bins above
10% peak occupy 40–270 ms, versus 10–160 ms for the reference. One ablation keeps
every tissue/tract setting and the peak-pressure parameter fixed, searches only
onset/attack/decay plus an optional respiratory-drive cutoff, and preserves the
previous features, loss and weights. `--render` remains reference-free. Older
15-control files retain the exact 360 ms cutoff and bit-identical output.

573 evaluations, no mechanical failures: envelope RMS .2191 → .1641, spectral RMS
8.1390 → 7.6865 dB, periodicity RMS .1157 → .1003, combined objective 86.8264 →
73.1717. Strong-energy extent becomes 10–200 ms. However, the peak moves from 70 ms
to 30 ms against a 60 ms target; attack reaches the 4 ms lower bound. This is an
acoustic tradeoff, not a validated better bark. Keeping the retuned timing while
restoring cutoff to 360 ms yields envelope RMS .1647, so most envelope gain is due
to onset/attack retiming rather than the added cutoff. The user rejected the voice
as blown-air/windpipe timbre, without convincing bark or howl. The trial is rejected
at listening despite improved calibration metrics. No preset is promoted.

The local listening review at http://127.0.0.1:58071/dog-pulse/ includes separate
reference/physical clips, timing plots and the matched-control cutoff ablation.
All clips are peak-normalized, not perceptually loudness-matched. The benchmark
and candidate configuration remain outside Git with explicit provenance.

The [continuum reproduction specification](dog-continuum.json) separately records
published canine inputs, unresolved data and proposed implementation gates. The
current four-mass source is not a reproduction of that continuum study; neither
fitting an isolated bark nor importing elastic moduli establishes one.

## Source isolation after listening rejection

```sh
cargo run --release -p chimera --example audit_dog_source -- ../research/dog-pulse/source-audit
```

The audit crosses fold vibration enabled/frozen with turbulence enabled/disabled,
using the exact default live Bark/Howl plans. The frozen diagnostic bypasses tissue
stepping and sets the glottal port to its configured rest area; it retains breath,
airway articulation and flow. It is not an alternative animal source. Within each
call, every WAV uses one common gain, so a near-silent case is not amplified into
a plausible tone. Audio-rate runs also expose actual aperture/flow traces sampled
at 12 kHz for two 40 ms windows. Reading source state leaves the output unchanged.

Howl RMS is .031852 with turbulence and .031856 without; estimated fundamental
328.660/328.661 Hz. Freezing the tissue reduces sustained RMS to 7.66e-7. Bark RMS
is .060275/.060387 with/without turbulence, versus .002557 with frozen rest aperture.
Thus the sustained tone depends on tissue motion, not mainly the additive noise
source. These nonlinear interventions are not an additive energy decomposition,
and pitch estimates on the near-silent cases are meaningless. The exact evidence
rules out a simple hiss-level explanation; it does not prove a specific corrective
mechanism or that a particular higher-fidelity model will sound canine.

Further envelope optimization is stopped. The next research gate is canine source
validation: body/cover geometry, constitutive properties, glottal flow and mucosal
motion before natural-bark fitting. The published continuum benchmark and any
reduced model derived from it remain separate from the rejected current source.

Source documentation created by Codex GPT-6, session /root, 2026-11-09 local date,
2026-11-09 UTC; owner cdjk. Status active for physical bark rework; retain while the
experiment is used. Review after listening feedback, next review 2026-11-10. Project
MIT license applies to original code; bundled references retain CC0 provenance in
creatures/data/README.md. No training and no HF upload. Local artifact provenance and
review inventory are under research/dog-rework/README.md.
