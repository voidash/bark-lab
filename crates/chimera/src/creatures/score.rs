//! Local user scores, validated before installing them on a creature's audio clock.
//! Loading allocates; evaluating/rendering the compiled CallPlan does not.
use super::types::{CallPlan, CallTarget, CreatureDef, Curve, PlanKey};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const SCORE_ID: &str = "score";
pub(super) const SCORE_CALL: usize = usize::MAX;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureScoreKey {
    pub t: f64,
    pub values: BTreeMap<String, f64>,
    #[serde(default)]
    pub curve: Curve,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureScore {
    pub keys: Vec<CreatureScoreKey>,
}

impl CreatureScore {
    pub(super) fn compile(&self, def: &CreatureDef) -> Result<CallPlan, String> {
        if !(2..=2000).contains(&self.keys.len()) {
            return Err("score requires 2–2000 keys".into());
        }
        let mut plan = CallPlan {
            owns_mask: vec![false; def.params.len()],
            ..Default::default()
        };
        let mut previous = -1.0;
        for (ki, key) in self.keys.iter().enumerate() {
            if !key.t.is_finite()
                || !(0.0..=120.0).contains(&key.t)
                || key.t < previous
                || (ki == 0 && key.t != 0.0)
                || key.values.is_empty()
                || key.values.len() > def.params.len() + def.events.len() + 1
            {
                return Err(format!("invalid time/values at score key {ki}"));
            }
            previous = key.t;
            let mut values = Vec::new();
            for (name, &value) in &key.values {
                if !value.is_finite() {
                    return Err(format!("non-finite score value {name}"));
                }
                let target = if name == "gate" {
                    if !(0.0..=1.5).contains(&value) {
                        return Err("score gate outside 0–1.5".into());
                    }
                    CallTarget::Gate
                } else if let Some(event) = name.strip_prefix('!') {
                    let i = def
                        .events
                        .iter()
                        .position(|e| *e == event)
                        .ok_or_else(|| format!("unknown score event {name}"))?;
                    if !(0.0..=1.5).contains(&value) {
                        return Err(format!("score event amount outside 0–1.5: {name}"));
                    }
                    CallTarget::Event(Some(i))
                } else {
                    let i = def
                        .param_index(name)
                        .ok_or_else(|| format!("unknown score parameter {name}"))?;
                    let p = &def.params[i];
                    if !(p.min..=p.max).contains(&value) {
                        return Err(format!("score {name} outside {}–{}", p.min, p.max));
                    }
                    if !plan.owns_mask[i] {
                        plan.owns.push(i);
                        plan.owns_mask[i] = true;
                    }
                    CallTarget::Param(i)
                };
                if !plan.names.contains(&target) {
                    plan.names.push(target);
                }
                values.push((target, value));
            }
            plan.keys.push(PlanKey {
                t: key.t,
                v: values,
                curve: Some(key.curve),
            });
        }
        if self.keys.last().and_then(|k| k.values.get("gate")) != Some(&0.0) {
            return Err("score must end with gate zero".into());
        }
        Ok(plan)
    }
}
