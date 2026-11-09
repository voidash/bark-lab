//! Air-driven, bilateral two-mass vocal folds (four mechanical degrees of freedom).
//!
//! Starting equations: Steinecke–Herzel, as documented in Neubauer (2004), §2.5.1,
//! https://edoc.hu-berlin.de/bitstreams/b1b3ad61-9eef-444a-8dd7-b9d0e2b39719/download
//! Unlike ParametricGlottis, there is no phase accumulator or prescribed pulse. Lower and
//! upper edges move under pressure, spring, damping and inter-fold collision forces.
//! The host supplies instantaneous sub/supraglottal pressures and uses the resulting
//! minimum aperture in its acoustic flow solver. This is a reduced mammalian model;
//! these lumped tissue parameters are NOT measurements of a particular dog.

use crate::math::sqrt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldError {
    InvalidInput,
    UnstableState,
}

#[derive(Clone, Debug)]
pub struct TwoMassFolds {
    // [left lower x,v, left upper x,v, right lower x,v, right upper x,v], CGS.
    state: [f64; 8],
    dt: f64,
    tension: f64,
    asymmetry: f64,
    rest_area: f64,
    damping: f64,
    pub area: f64,
}

impl TwoMassFolds {
    pub fn new(fs: f64) -> Self {
        assert!(
            fs.is_finite() && fs >= 8000.0,
            "two-mass folds need a finite audio rate >= 8 kHz"
        );
        Self {
            state: [0.001, 0.0, 0.0, 0.0, 0.001, 0.0, 0.0, 0.0],
            dt: 1.0 / fs,
            tension: 1.0,
            asymmetry: 1.0,
            rest_area: 0.05,
            damping: 1.0,
            area: 0.05,
        }
    }

    /// Tension Q scales stiffness by Q and mass by 1/Q. Asymmetry detunes the right
    /// fold mechanically; it does not add jitter, random excitation or a second audio signal.
    pub fn configure(
        &mut self,
        tension: f64,
        asymmetry: f64,
        rest_area: f64,
        damping: f64,
    ) -> Result<(), FoldError> {
        if ![tension, asymmetry, rest_area, damping]
            .iter()
            .all(|x| x.is_finite())
            || !(0.3..=8.0).contains(&tension)
            || !(0.5..=2.0).contains(&asymmetry)
            || !(0.001..=0.2).contains(&rest_area)
            || !(0.3..=3.0).contains(&damping)
        {
            return Err(FoldError::InvalidInput);
        }
        self.tension = tension;
        self.asymmetry = asymmetry;
        self.rest_area = rest_area;
        self.damping = damping;
        Ok(())
    }

    fn derivative(&self, s: &[f64; 8], sub: f64, supra: f64) -> [f64; 8] {
        let length = 1.4;
        let a1 = self.rest_area + length * (s[0] + s[4]);
        let a2 = self.rest_area + length * (s[2] + s[6]);
        let minimum = a1.min(a2).max(0.0);
        let dp = sub - supra;
        let p1 = if a1 > 0.0 {
            supra + dp * (1.0 - (minimum / a1).powi(2))
        } else {
            sub
        };
        let p2 = if a1 > 0.0 && a2 <= 0.0 {
            sub
        } else if a1 > 0.0 {
            supra
        } else {
            0.0
        };
        let mut out = [0.0; 8];
        for side in 0..2 {
            let q = self.tension * if side == 0 { 1.0 } else { self.asymmetry };
            let base = side * 4;
            let coupling = 25000.0 * q;
            for edge in 0..2 {
                let i = base + edge * 2;
                let j = base + (1 - edge) * 2;
                let (mass, spring, depth, pressure, aperture) = if edge == 0 {
                    (0.125 / q, 80000.0 * q, 0.25, p1, a1)
                } else {
                    (0.025 / q, 8000.0 * q, 0.05, p2, a2)
                };
                let x = s[i];
                let v = s[i + 1];
                // Both folds share contact penetration; force is equal and opposite in
                // global coordinates (the two local x axes both point away from midline).
                let mean_q = self.tension * (1.0 + self.asymmetry) * 0.5;
                let contact_stiffness = if edge == 0 { 80000.0 } else { 8000.0 } * mean_q;
                let contact = 3.0 * contact_stiffness * (-aperture / (2.0 * length)).max(0.0);
                let damping = 20.0 * self.damping * if aperture < 0.0 { 3.0 } else { 1.0 };
                let restoring = spring * x * (1.0 + 100.0 * x * x);
                out[i] = v;
                out[i + 1] = (length * depth * pressure + contact
                    - restoring
                    - damping * v
                    - coupling * (x - s[j]))
                    / mass;
            }
        }
        out
    }

    /// RK4 with a mechanical step no longer than 1/96 kHz. The acoustic boundary
    /// pressure is held for this sample; the host updates it on the next acoustic step.
    pub fn step(&mut self, sub: f64, supra: f64) -> Result<f64, FoldError> {
        if !sub.is_finite() || !supra.is_finite() {
            return Err(FoldError::InvalidInput);
        }
        let steps = (self.dt * 96000.0).ceil() as usize;
        let dt = self.dt / steps as f64;
        for _ in 0..steps {
            let s = self.state;
            let k1 = self.derivative(&s, sub, supra);
            let mut mid = [0.0; 8];
            for i in 0..8 {
                mid[i] = s[i] + 0.5 * dt * k1[i];
            }
            let k2 = self.derivative(&mid, sub, supra);
            for i in 0..8 {
                mid[i] = s[i] + 0.5 * dt * k2[i];
            }
            let k3 = self.derivative(&mid, sub, supra);
            for i in 0..8 {
                mid[i] = s[i] + dt * k3[i];
            }
            let k4 = self.derivative(&mid, sub, supra);
            for i in 0..8 {
                self.state[i] = s[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            }
        }
        if self.state.iter().any(|x| !x.is_finite())
            || [0, 2, 4, 6].iter().any(|&i| self.state[i].abs() > 1.0)
        {
            // Do not silently reset a diverging oscillator and call it a stable model.
            return Err(FoldError::UnstableState);
        }
        self.area = (self.rest_area + 1.4 * (self.state[0] + self.state[4]))
            .min(self.rest_area + 1.4 * (self.state[2] + self.state[6]))
            .max(0.0);
        Ok(self.area)
    }

    /// Instantaneous left lower/upper and right lower/upper displacements, in cm.
    /// This is a snapshot of the solved tissue state, not a prescribed cycle.
    pub fn displacements(&self) -> [f64; 4] {
        [self.state[0], self.state[2], self.state[4], self.state[6]]
    }

    /// Total resting aperture, in cm², shared between the left and right folds.
    pub fn rest_area(&self) -> f64 {
        self.rest_area
    }

    /// Effective fold length in this reduced model, in cm; not a breed measurement.
    pub fn length(&self) -> f64 {
        1.4
    }

    /// Tissue speed, useful for source telemetry; no assumed cyclic phase.
    pub fn speed(&self) -> f64 {
        sqrt(
            [1, 3, 5, 7]
                .iter()
                .map(|&i| self.state[i] * self.state[i])
                .sum::<f64>(),
        )
    }
}
