# Canine-informed body–cover candidate

Created 2026-11-09 UTC by Codex GPT-6, session /root, owner cdjk, in
/Users/cdjk/github/dog/chimera-vocal-synth. Active documentation for the candidate
branch; review after the owner listens. Original code is MIT; cited papers retain
their rights. No new dataset, weights or HF repository.

Review the actual lab at http://127.0.0.1:5318/?tissue=canine. Its source selector
compares the new six-mass model with the rejected legacy four-mass model.
The baseline production build is temporarily at http://127.0.0.1:5319/.
Neither is a validated bark synthesizer. The legacy model remains the engine
default pending listening. The query selects the candidate explicitly.

## What changed

Each fold now has two cover edges and an independently moving body, rather than
two edges attached to a fixed foundation. Cover/body springs and viscous links
exert equal and opposite internal forces. The bodies have their own foundation
springs; contact prevents unrestricted interpenetration of opposing covers.
RK4 advances all six masses. The minimum opening drives the acoustic port, and
the tracheal/pharyngeal pressures feed back into the tissue. There is no audio
oscillator, reference playback or output timbre effect in this source.

[Jiang et al.](https://pmc.ncbi.nlm.nih.gov/articles/PMC8547019/) supplies a 12.5 mm
length, 3.5 mm medial height, density 1040 kg/m³, body transverse modulus 4.76 kPa,
superior cover transverse modulus 1.33 kPa with inferior gradient 0.43 kPa/mm,
and longitudinal moduli 95.24/26.70 kPa. These are inputs to a new lumped reduction,
not a reproduction of that paper's finite-element model. Its experiment used an
excised canine larynx without the whole supraglottal tract.

Lateral cover/body depths of 0.5/2.5 mm are assumptions. Mass is density × cell
volume. Cross-plane spring stiffness is E × loaded area / depth. The cover
cells sample the modulus gradient at their centres. Vertical coupling uses a
plane shear approximation; this omits the full anisotropic constitutive tensor.
The reported transverse Poisson ratio 0.9 is not treated as an isotropic 3D ratio.

The nominal pitch control adds longitudinal prestress stiffness with fixed tissue
mass. It subtracts a body-mode estimate before adding stiffness; this mapping is
an engineering controller, not a measured muscle activation law or calibrated
pitch. Shared longitudinal strain gives the cover/body stress ratio 26.70/95.24.
An initial equal-stress assumption failed to phonate around nominal 330 Hz and
was discarded. Damping ratios (body 0.2, cover 0.12), contact stiffness and left/right
detuning are also assumptions requiring sensitivity tests and validation.

The aerodynamic approximation is a quasi-steady separated jet: the lower open
surface sees the pressure drop set by the minimum opening, and the upper open
surface sees supraglottal pressure. Closed medial surfaces are shielded. This
does not resolve vortices, jet attachment, spatially distributed pressures or
energy-consistent unsteady flow/structure coupling. Acoustic port flow remains
inertive/resistive. Those limits matter when assessing a rough bark.

## What the tests establish

Pressure can sustain tissue vibration; unpowered motion decays. Increased
prestress raises the measured tissue frequency without changing mass. Live calls
have moving bodies and covers, nonzero flow, the exact geometric aperture, finite
output, and release to silence. Block sizes 77, 128 and 4096 give identical bark
output. Individual parameter endpoints are checked at 44.1, 48 and 96 kHz.
These checks establish numerical and causal behaviour, not dog likeness.

`audit_dog_source OUTPUT --canine` renders the same live Bark/Howl plans with
folds moving/frozen and acoustic turbulence enabled/disabled. Gains are shared
within each call's ablation group. Compare raw RMS across source versions;
separately normalized WAVs are not loudness-matched between versions.
`probe_canine_cover` isolates tissue and reports onset, frequency and decay;
`probe_dog_regimes --focus --canine` explores a bounded interaction grid.

The 54-case grid had no faults and included both periodic and irregular regimes.
The explicit **Rough bark test** uses nominal 330 Hz, 24 cmH₂O, detuning 0.85,
damping multiplier 0.7 and throat area 0.3 cm² with a short pressure/mouth gesture.
These are effective performance controls, not measured canine articulations.
Its early-window YIN aperiodicity was 0.625, or 0.554 with turbulence disabled;
the corresponding window RMS was 0.113/0.116. Freezing the folds reduced RMS to
0.00148. The clean body–cover howl had aperiodicity 0.00071. These measurements
support contrasting tissue regimes, not a perceptual claim that either is a dog.

Five narrower legacy pressure/contact corrections were tested and rejected for
losing its guarded irregular regime. They were reverted; legacy equations and
judges remain unchanged. Patches and compact results are inventoried under
../research/dog-flow for this review, with timed retention.

## Remaining work

The owner must judge whether the new source improves the bark/howl. A numerical
pass does not authorize calling it realistic. The new tissue also changes the
relationship between nominal note, pressure and phonation, so legacy musical
gestures need re-identification rather than assuming their old scaling applies.
MRI segmentation, continuum tissue mechanics and Navier–Stokes FSI remain a
separate, uncompleted reproduction task; there is no MRI-derived mesh here.

Reproduce with the Rust examples above and `bun run build`; select the candidate
with the query or lab selector. The lab visualizes the actual solver's cover and
body displacement, magnified, sampled at UI rate. It is not a slowed anatomical
movie or a substitute for medical imaging.
