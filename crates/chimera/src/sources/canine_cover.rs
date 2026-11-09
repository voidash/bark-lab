//! Reduced canine body-cover candidate: two cover edges and one body per fold.
//!
//! L=12.5 mm, vertical thickness=3.5 mm, density=1040 kg/m³ and body/cover
//! transverse moduli/gradient come from Jiang et al., doi:10.1115/1.4052226.
//! Lateral depths (cover .5 mm/body 2.5 mm), damping ratios and the lumped reduction
//! are explicit engineering assumptions, NOT an MRI reconstruction or FSI reproduction.
//! Tissue mass follows rho*volume and does not shrink when pitch increases.
use crate::math::{PI, sqrt};
use crate::sources::two_mass::FoldError;

const LENGTH: f64 = 1.25; // cm
const HEIGHT: f64 = 0.35;
const COVER_DEPTH: f64 = 0.05; // assumed cm
const BODY_DEPTH: f64 = 0.25; // assumed cm
const DENSITY: f64 = 1.04; // g/cm³
const COVER_MASS: f64 = DENSITY * LENGTH * HEIGHT * 0.5 * COVER_DEPTH;
const BODY_MASS: f64 = DENSITY * LENGTH * HEIGHT * BODY_DEPTH;
const BODY_K: f64 = 47600.0 * LENGTH * HEIGHT / BODY_DEPTH;
// Cell-centre moduli: 1.33 kPa at superior edge + .43 kPa/mm inferior gradient.
const COVER_K: [f64; 2] = [
    24587.5 * LENGTH * HEIGHT * 0.5 / COVER_DEPTH,
    17062.5 * LENGTH * HEIGHT * 0.5 / COVER_DEPTH,
];
const SHEAR_K: f64 = (20825.0 / 3.8) * LENGTH * COVER_DEPTH / (HEIGHT * 0.5);
// Shared longitudinal strain produces different body/cover prestress according
// to the published longitudinal moduli, rather than imposing equal modal shifts.
const COVER_LONGITUDINAL_RATIO: f64 = 26.70 / 95.24;

#[derive(Clone, Debug)]
pub struct CanineCover {
    // left lower/upper/body x,v, followed by right lower/upper/body; cm and cm/s.
    state: [f64; 12],
    dt: f64,
    frequency: f64,
    asymmetry: f64,
    rest_area: f64,
    damping: f64,
    pub area: f64,
}

impl CanineCover {
    pub fn new(fs: f64) -> Self {
        assert!(
            fs.is_finite() && fs >= 8000.0,
            "canine tissue needs finite fs >= 8 kHz"
        );
        Self {
            state: [
                0.0001, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0001, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            dt: 1.0 / fs,
            frequency: 170.0,
            asymmetry: 1.0,
            rest_area: 0.05,
            damping: 1.0,
            area: 0.05,
        }
    }

    pub fn configure(
        &mut self,
        frequency: f64,
        asymmetry: f64,
        rest_area: f64,
        damping: f64,
    ) -> Result<(), FoldError> {
        if ![frequency, asymmetry, rest_area, damping]
            .iter()
            .all(|v| v.is_finite())
            || !(50.0..=1400.0).contains(&frequency)
            || !(0.5..=2.0).contains(&asymmetry)
            || !(0.001..=0.2).contains(&rest_area)
            || !(0.3..=3.0).contains(&damping)
        {
            return Err(FoldError::InvalidInput);
        }
        self.frequency = frequency;
        self.asymmetry = asymmetry;
        self.rest_area = rest_area;
        self.damping = damping;
        Ok(())
    }

    fn derivative(&self, s: &[f64; 12], sub: f64, supra: f64) -> [f64; 12] {
        let areas = [
            self.rest_area + LENGTH * (s[0] + s[6]),
            self.rest_area + LENGTH * (s[2] + s[8]),
        ];
        let minimum = areas[0].min(areas[1]).max(0.0);
        let pressure = [
            if areas[0] > 0.0 {
                supra + (sub - supra) * (1.0 - (minimum / areas[0]).powi(2))
            } else {
                0.0
            },
            if areas[1] > 0.0 { supra } else { 0.0 },
        ];
        // Longitudinal prestress raises modal stiffness, not a prescribed oscillation.
        // Nominal pitch subtracts a body-mode estimate; frequency still emerges from FSI.
        let base = BODY_K / (BODY_MASS + 2.0 * COVER_MASS);
        let stress = ((2.0 * PI * self.frequency).powi(2) - base).max(0.0);
        let mut out = [0.0; 12];
        for side in 0..2 {
            let b = side * 6;
            let q = if side == 0 { 1.0 } else { self.asymmetry };
            let mut body_force = -q * BODY_K * s[b + 4]
                - 2.0 * 0.2 * self.damping * sqrt(BODY_K * BODY_MASS) * s[b + 5]
                - q * BODY_MASS * stress * s[b + 4];
            for edge in 0..2 {
                let i = b + edge * 2;
                let other = b + (1 - edge) * 2;
                let k = q * COVER_K[edge];
                let visc = 2.0 * 0.12 * self.damping * sqrt(k * COVER_MASS);
                let link = k * (s[i] - s[b + 4]) + visc * (s[i + 1] - s[b + 5]);
                let shear = q * SHEAR_K * (s[i] - s[other]);
                let penetration = (-areas[edge] / (2.0 * LENGTH)).max(0.0);
                // Equal and opposite physical contact in global coordinates.
                let effective = 2.0 * COVER_K[edge] * self.asymmetry / (1.0 + self.asymmetry);
                let contact = 3.0 * effective * penetration;
                let force = LENGTH * HEIGHT * 0.5 * pressure[edge] + contact
                    - link
                    - shear
                    - q * COVER_MASS * stress * COVER_LONGITUDINAL_RATIO * s[i];
                out[i] = s[i + 1];
                out[i + 1] = force / COVER_MASS;
                body_force += link;
            }
            out[b + 4] = s[b + 5];
            out[b + 5] = body_force / BODY_MASS;
        }
        out
    }

    pub fn step(&mut self, sub: f64, supra: f64) -> Result<f64, FoldError> {
        if ![sub, supra].iter().all(|v| v.is_finite()) {
            return Err(FoldError::InvalidInput);
        }
        let steps = (self.dt * 192000.0).ceil() as usize;
        let dt = self.dt / steps as f64;
        for _ in 0..steps {
            let s = self.state;
            let k1 = self.derivative(&s, sub, supra);
            let mut mid = [0.0; 12];
            for i in 0..12 {
                mid[i] = s[i] + 0.5 * dt * k1[i];
            }
            let k2 = self.derivative(&mid, sub, supra);
            for i in 0..12 {
                mid[i] = s[i] + 0.5 * dt * k2[i];
            }
            let k3 = self.derivative(&mid, sub, supra);
            for i in 0..12 {
                mid[i] = s[i] + dt * k3[i];
            }
            let k4 = self.derivative(&mid, sub, supra);
            for i in 0..12 {
                self.state[i] = s[i] + dt * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0;
            }
        }
        if self.state.iter().any(|v| !v.is_finite())
            || [0, 2, 4, 6, 8, 10]
                .iter()
                .any(|&i| self.state[i].abs() > 1.0)
        {
            return Err(FoldError::UnstableState);
        }
        self.area = (self.rest_area + LENGTH * (self.state[0] + self.state[6]))
            .min(self.rest_area + LENGTH * (self.state[2] + self.state[8]))
            .max(0.0);
        Ok(self.area)
    }

    pub fn displacements(&self) -> [f64; 4] {
        [self.state[0], self.state[2], self.state[6], self.state[8]]
    }
    pub fn body_displacements(&self) -> [f64; 2] {
        [self.state[4], self.state[10]]
    }
    pub fn rest_area(&self) -> f64 {
        self.rest_area
    }
    pub fn length(&self) -> f64 {
        LENGTH
    }
    pub fn speed(&self) -> f64 {
        sqrt(
            [1, 3, 5, 7, 9, 11]
                .iter()
                .map(|&i| self.state[i] * self.state[i])
                .sum(),
        )
    }
}
