//! Physical canine candidate. Recordings are comparison targets only, never render inputs.
use super::build::{CMH2O, fill_areas};
use super::dog;
use super::mammal::{MammalAirway, mammal_noise};
use super::types::{CreatureDef, CreatureSim, ParamDef};
use crate::airway::network::{AirwayNetwork, Tube};
use crate::math::{PI, sin};
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

struct PhysicalDog {
    a: MammalAirway,
    folds: TwoMassFolds,
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
        if let Err(error) = self.folds.configure(
            p[0] / 170.0,
            1.0 - 0.55 * p[2],
            (0.045 - 0.03 * p[3] - 0.01 * p[2].abs()) * p[GAP],
            (0.9 - 0.2 * p[3] - 0.25 * p[2].abs()) * p[5],
        ) {
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
                if u < 0.3 {
                    p[7] + (1.2 + sin(PI * 0.3) - p[7]) * (u / 0.3)
                } else {
                    1.2 + sin(PI * u)
                }
            },
            &mut a.phar_area,
        );
        a.net
            .set_tube_targets(a.pharynx, ph, Some(6.0 * p[TRACT_LENGTH] / 17.0), None);
        let mouth = 0.12 + 3.5 * p[1];
        let oral = fill_areas(
            a.oral,
            |u| {
                if u < 0.65 {
                    p[CAVITY] * (2.6 + 0.6 * sin(PI * u / 0.65))
                } else {
                    2.6 * p[CAVITY] + (mouth - 2.6 * p[CAVITY]) * (u - 0.65) / 0.35
                }
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
        match self.folds.step(sub, supra) {
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
        out.push(("glottis", (self.folds.speed() / 100.0).min(1.0)));
        // A fault is latched and exposed, never hidden by resetting the mechanical state.
        out.push(("fault", if self.fault.is_some() { 1.0 } else { 0.0 }));
        let [left_lower, left_upper, right_lower, right_upper] = self.folds.displacements();
        out.extend([
            ("leftLower", left_lower),
            ("leftUpper", left_upper),
            ("rightLower", right_lower),
            ("rightUpper", right_upper),
            ("glottalArea", self.folds.area),
            ("restArea", self.folds.rest_area()),
            ("foldLength", self.folds.length()),
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
    def.blurb = "Pressure drives four coupled vocal-fold masses, collisions and the acoustic airway. No recordings play in this voice. Experimental canine timbre; pitch is nominal, not yet calibrated.";
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
    ]);
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
    def.create = create;
    def.resolved()
}
