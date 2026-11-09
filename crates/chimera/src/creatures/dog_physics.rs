//! Physical canine candidate. Recordings are comparison targets only, never render inputs.
use super::build::{CMH2O, fill_areas};
use super::dog;
use super::mammal::{MammalAirway, mammal_noise};
use super::types::{CallDef, CallKey, CreatureDef, CreatureSim, Curve, ParamDef};
use crate::airway::network::{AirwayNetwork, Tube};
use crate::math::{PI, sin};
use crate::sources::canine_cover::CanineCover;
use crate::sources::two_mass::{FoldError, TwoMassFolds};

/// Candidate irregular-phonation regime found by probe_dog_regimes. It must still
/// pass the listening comparison; these are effective controls, not breed measurements.
pub const BARK_SETTINGS: [(&str, f64); 5] = [
    ("tone", 0.0),
    ("rough", 0.85),
    ("pressure", 28.0),
    ("damping", 0.7),
    ("throat", 0.4),
];

const TRACT_LENGTH: usize = 8;
const CAVITY: usize = 9;
const ASPIRATION: usize = 10;
const GAP: usize = 11;
const TONGUE_HEIGHT: usize = 12;
const TONGUE_POSITION: usize = 13;
const TONGUE_ROOT: usize = 14;
const LARYNX_DROP: usize = 15;
const SOURCE_MODEL: usize = 16;

struct PhysicalDog {
    a: MammalAirway,
    folds: TwoMassFolds,
    cover: CanineCover,
    use_cover: bool,
    fs: f64,
    tubes: Vec<(&'static str, Tube)>,
    fault: Option<FoldError>,
}

fn create(fs: f64) -> Box<dyn CreatureSim> {
    let a = dog::airway(fs);
    let mut tubes = vec![
        ("trachea", a.trachea),
        ("pharynx", a.pharynx),
        ("oral", a.oral),
    ];
    if let Some(nasal) = a.nasal {
        tubes.push(("nasal", nasal));
    }
    Box::new(PhysicalDog {
        a,
        tubes,
        folds: TwoMassFolds::new(fs),
        cover: CanineCover::new(fs),
        use_cover: false,
        fs,
        fault: None,
    })
}

impl CreatureSim for PhysicalDog {
    fn net(&self) -> Option<&AirwayNetwork> {
        Some(&self.a.net)
    }
    fn net_mut(&mut self) -> Option<&mut AirwayNetwork> {
        Some(&mut self.a.net)
    }
    fn tubes(&self) -> &[(&'static str, Tube)] {
        &self.tubes
    }
    fn gain(&self) -> f64 {
        6.0e-8
    }

    fn control(&mut self, p: &[f64], gate: f64, _dt: f64) {
        // These are initial reduced-model controls, not an anatomical inversion. Frequency
        // emerges from the tissue/airway system and can depart from the nominal key pitch.
        let use_cover = p[SOURCE_MODEL] >= 0.5;
        if use_cover != self.use_cover {
            // Explicit model selection starts the selected tissue at rest. This
            // is not an automatic recovery from a fault: faults remain latched.
            if use_cover {
                self.cover = CanineCover::new(self.fs);
            } else {
                self.folds = TwoMassFolds::new(self.fs);
            }
            self.use_cover = use_cover;
        }
        let asymmetry = 1.0 - 0.55 * p[2];
        let rest = (0.045 - 0.03 * p[3] - 0.01 * p[2].abs()) * p[GAP];
        let damping = (0.9 - 0.2 * p[3] - 0.25 * p[2].abs()) * p[5];
        let result = if self.use_cover {
            self.cover.configure(p[0], asymmetry, rest, damping)
        } else {
            self.folds.configure(p[0] / 170.0, asymmetry, rest, damping)
        };
        if let Err(error) = result {
            self.fault = Some(error);
        }
        let a = &mut self.a;
        a.net.set_fixed_pressure(a.lungs, p[4] * CMH2O * gate);
        if let Some(velum) = a.velum {
            a.net.set_port_target(velum, p[6] * 0.5);
        }
        let ph = fill_areas(
            a.pharynx,
            |u| {
                // A finite throat entrance changes acoustic impedance and therefore the
                // pressure fed back to the tissue. No artificial feedback gain is applied.
                let area = if u < 0.3 {
                    p[7] + (1.2 + sin(PI * 0.3) - p[7]) * (u / 0.3)
                } else {
                    1.2 + sin(PI * u)
                };
                // Effective tongue-root retraction reduces the oropharyngeal
                // lumen. The 1-D area mapping is an engineering approximation,
                // not a segmented midsagittal slice or a muscle simulation.
                area * (1.0 - 0.55 * p[TONGUE_ROOT] * sin(PI * u).powi(2))
            },
            &mut a.phar_area,
        );
        a.net.set_tube_targets(
            a.pharynx,
            ph,
            Some(6.0 * p[TRACT_LENGTH] / 17.0 + p[LARYNX_DROP]),
            None,
        );
        let mouth = 0.12 + 3.5 * p[1];
        let oral = fill_areas(
            a.oral,
            |u| {
                let area = if u < 0.65 {
                    p[CAVITY] * (2.6 + 0.6 * sin(PI * u / 0.65))
                } else {
                    2.6 * p[CAVITY] + (mouth - 2.6 * p[CAVITY]) * (u - 0.65) / 0.35
                };
                let distance = ((u - p[TONGUE_POSITION]) / 0.30).abs().min(1.0);
                let constriction = 1.0 - distance * distance;
                area * (1.0 - 0.75 * p[TONGUE_HEIGHT] * constriction * constriction)
            },
            &mut a.oral_area,
        );
        a.net
            .set_tube_targets(a.oral, oral, Some(11.0 * p[TRACT_LENGTH] / 17.0), None);
        mammal_noise(a, p[ASPIRATION] * (1.0e-6 + 5.0e-6 * (1.0 - p[3])), None);
    }

    fn sample(&mut self) -> f64 {
        if self.fault.is_some() {
            return 0.0;
        }
        let sub = self.a.net.p[self.a.trachea.last_node()];
        let supra = self.a.net.p[self.a.pharynx.first_node()];
        let result = if self.use_cover {
            self.cover.step(sub, supra)
        } else {
            self.folds.step(sub, supra)
        };
        match result {
            Ok(area) => self.a.net.set_port_area(self.a.glottis, area),
            Err(error) => {
                self.fault = Some(error);
                return 0.0;
            }
        }
        self.a.net.step()
    }

    fn source_state(&self, out: &mut Vec<(&'static str, f64)>) {
        out.clear();
        let speed = if self.use_cover {
            self.cover.speed()
        } else {
            self.folds.speed()
        };
        out.push(("glottis", (speed / 100.0).min(1.0)));
        // A fault is latched and exposed, never hidden by resetting the mechanical state.
        out.push(("fault", if self.fault.is_some() { 1.0 } else { 0.0 }));
        let [left_lower, left_upper, right_lower, right_upper] = if self.use_cover {
            self.cover.displacements()
        } else {
            self.folds.displacements()
        };
        let body = if self.use_cover {
            self.cover.body_displacements()
        } else {
            [0.0, 0.0]
        };
        out.extend([
            ("leftLower", left_lower),
            ("leftUpper", left_upper),
            ("rightLower", right_lower),
            ("rightUpper", right_upper),
            (
                "glottalArea",
                if self.use_cover {
                    self.cover.area
                } else {
                    self.folds.area
                },
            ),
            (
                "restArea",
                if self.use_cover {
                    self.cover.rest_area()
                } else {
                    self.folds.rest_area()
                },
            ),
            (
                "foldLength",
                if self.use_cover {
                    self.cover.length()
                } else {
                    self.folds.length()
                },
            ),
            ("sourceModel", if self.use_cover { 1.0 } else { 0.0 }),
            ("leftBody", body[0]),
            ("rightBody", body[1]),
            ("glottalFlow", self.a.net.u[self.a.glottis]),
            ("lungPressure", self.a.net.p[self.a.lungs] / CMH2O),
            (
                "subPressure",
                self.a.net.p[self.a.trachea.last_node()] / CMH2O,
            ),
            (
                "supraPressure",
                self.a.net.p[self.a.pharynx.first_node()] / CMH2O,
            ),
            (
                "tracheaLength",
                self.a.net.tube_length[self.a.trachea.index],
            ),
            (
                "pharynxLength",
                self.a.net.tube_length[self.a.pharynx.index],
            ),
            ("oralLength", self.a.net.tube_length[self.a.oral.index]),
        ]);
        if let Some(nasal) = self.a.nasal {
            out.push(("nasalLength", self.a.net.tube_length[nasal.index]));
        }
    }
}

pub fn def() -> CreatureDef {
    let mut def = dog::def();
    def.id = "dog-physics";
    def.name = "Dog physics";
    def.latin = "Air-driven tissue model";
    def.blurb = "Breath drives vocal-fold tissue, collisions and the acoustic airway. Compare the legacy source with a six-mass canine-informed body–cover candidate. Pitch is nominal; dog likeness still needs listening validation.";
    def.role = "Physical vocal folds + airway · no recorded excitation";
    def.params[0].label = "Nominal pitch";
    def.params[0].max = 1000.0;
    def.params[2].label = "Fold detuning";
    def.params[2].min = -0.85;
    def.params[5].key = "damping";
    def.params[5].label = "Tissue damping";
    def.params[5].min = 0.7;
    def.params[5].max = 1.8;
    def.params[5].default = 1.0;
    def.params[4].default = 16.0;
    def.params[4].max = 48.0;
    def.params.push(
        ParamDef::new("throat", "Throat entrance", 0.18, 1.8, 1.2)
            .unit("cm²")
            .smooth(0.02),
    );
    // Inverse fitting needs independent source and filter controls. The original
    // model hid these at one arbitrary setting, so roughness alone could not fit
    // a bark's resonances, spectral tilt, or glottal closure. Length is restricted
    // to an adult-dog subset of Riede & Fitch (1999)'s 6.9–22.4 cm measurements.
    // Never shorten this existing grid below 17 cm: that would violate its CFL
    // design. These area/gap bounds are engineering search bounds, not measured
    // anatomy. A fitted acoustic match is not a unique anatomical reconstruction.
    def.params.extend([
        ParamDef::new("tractLength", "Tract", 17.0, 22.4, 17.0)
            .unit("cm")
            .smooth(0.03),
        ParamDef::new("cavity", "Cavity", 0.4, 2.0, 1.0).smooth(0.02),
        ParamDef::new("aspiration", "Noise", 0.05, 2.0, 1.0).smooth(0.01),
        ParamDef::new("gap", "Gap", 0.4, 1.6, 1.0).smooth(0.008),
        ParamDef::new("tongueHeight", "Lift", 0.0, 1.0, 0.0).smooth(0.025),
        ParamDef::new("tonguePosition", "Pos.", 0.25, 0.80, 0.50).smooth(0.025),
        ParamDef::new("tongueRoot", "Root", 0.0, 1.0, 0.0).smooth(0.025),
        // Only lengthening is allowed on the fixed grid, preserving its CFL
        // condition. 0–2 cm is a design bound, not a measured bark trajectory.
        ParamDef::new("larynxDrop", "Drop", 0.0, 2.0, 0.0)
            .unit("cm")
            .smooth(0.03),
    ]);
    // Version choice, not a blend of oscillators. Legacy stays the default until
    // the new source passes listening; the lab exposes an explicit A/B selector.
    def.params.push(
        ParamDef::new("sourceModel", "Model", 0.0, 1.0, 0.0)
            .digits(0)
            .smooth(0.001),
    );
    for call in &mut def.calls {
        if call.id == "bark" || call.id == "barkTune" {
            for key in &mut call.keys {
                // Keep the old arrangement's notes, timing, breath and mouth gestures.
                // Only its physical phonation regime changes.
                if key.v.0.iter().any(|(name, _)| *name == "rough") {
                    for (name, value) in BARK_SETTINGS {
                        if let Some((_, old)) = key.v.0.iter_mut().find(|(k, _)| *k == name) {
                            *old = value;
                        } else {
                            key.v.0.push((name, value));
                        }
                    }
                }
                // At fixed dimensionless pressure P/Q, the tissue equations retain their
                // regime as tension Q changes. Coordinate actual breath with note pitch;
                // otherwise the upper notes fall back into clean phonation. Acoustic tract
                // resonances remain independent, so this is an initial controller, not an
                // exact invariance of the complete source–tract system.
                if let Some((_, pitch)) = key.v.0.iter().find(|(name, _)| *name == "pitch") {
                    let pressure = 28.0 * pitch / 330.0;
                    if let Some((_, value)) =
                        key.v.0.iter_mut().find(|(name, _)| *name == "pressure")
                    {
                        *value = pressure;
                    } else {
                        key.v.0.push(("pressure", pressure));
                    }
                }
            }
        }
    }
    // A separate, explicit experiment: the legacy gestures are not calibrated
    // for this tissue. The bounded body-cover grid found irregular phonation at
    // these effective controls. This is not a recording fit or a validated bark.
    def.calls.push(CallDef::new(
        "roughBark",
        "Rough bark test",
        "Six-mass body–cover source with a short breath pulse; experimental dog likeness",
        Some(330.0),
        vec![
            CallKey::new(
                0.0,
                &[
                    ("sourceModel", 1.0),
                    ("gate", 0.0),
                    ("pitch", 330.0),
                    ("pressure", 24.0),
                    ("rough", 0.85),
                    ("damping", 0.7),
                    ("tone", 0.0),
                    ("throat", 0.3),
                    ("mouth", 0.65),
                    ("aspiration", 0.05),
                    ("gap", 1.0),
                    ("nasal", 0.0),
                ],
            )
            .curve(Curve::Step),
            CallKey::new(0.012, &[("gate", 1.0)]),
            CallKey::new(0.18, &[("gate", 1.0), ("pitch", 330.0)]),
            CallKey::new(0.24, &[("gate", 0.7), ("pitch", 280.0), ("mouth", 0.3)]),
            CallKey::new(0.30, &[("gate", 0.0), ("mouth", 0.12)]),
        ],
    ));
    def.create = create;
    def.resolved()
}
