//! Render a local note/gesture score through dog-physics; never load any audio source.
use chimera::creatures::dog_physics::BARK_SETTINGS;
use chimera::creatures::{CreatureVoice, creature_by_id};
use chimera::math::exp;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Deserialize)]
struct Note {
    t: f64,
    duration: f64,
    pitch: f64,
    amplitude: f64,
}
#[derive(Deserialize)]
struct Score {
    title: String,
    notes: Vec<Note>,
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: render_dog_score score.json output.wav".into());
    }
    let input = std::fs::read_to_string(&args[0]).map_err(|e| format!("{}: {e}", args[0]))?;
    let score: Score = serde_json::from_str(&input).map_err(|e| format!("{}: {e}", args[0]))?;
    if score.notes.is_empty() || score.notes.len() > 5000 {
        return Err("score must contain 1–5000 notes".into());
    }
    let mut previous = -1.0;
    for n in &score.notes {
        if ![n.t, n.duration, n.pitch, n.amplitude]
            .iter()
            .all(|x| x.is_finite())
            || n.t < previous
            || !(0.0..=300.0).contains(&n.t)
            || !(0.02..=2.0).contains(&n.duration)
            || !(100.0..=1000.0).contains(&n.pitch)
            || !(0.0..=1.5).contains(&n.amplitude)
        {
            return Err("invalid or unsorted note: require finite t 0–300, duration .02–2, pitch 100–1000 and amplitude 0–1.5".into());
        }
        previous = n.t;
    }
    let def = creature_by_id("dog-physics").ok_or("physical dog is not registered")?;
    let param = |key| {
        def.param_index(key)
            .ok_or_else(|| format!("missing physical control {key}"))
    };
    let (pitch, mouth, rough, tone, pressure, damping) = (
        param("pitch")?,
        param("mouth")?,
        param("rough")?,
        param("tone")?,
        param("pressure")?,
        param("damping")?,
    );
    let mut voice = CreatureVoice::new(
        def,
        48000.0,
        &[
            (tone, 0.0),
            (rough, 0.62),
            (pressure, 22.0),
            (damping, 1.0),
            (mouth, 0.8),
        ],
    );
    for (key, value) in BARK_SETTINGS {
        voice.set_params(&[(param(key)?, value)]);
    }
    let duration = score
        .notes
        .iter()
        .map(|n| n.t + n.duration)
        .fold(0.0, f64::max)
        + 0.8;
    let count = (duration * 48000.0).ceil() as usize;
    let mut audio = vec![0.0f32; count];
    let mut next = 0;
    let mut active = None;
    let t0 = Instant::now();
    for (index, block) in audio.chunks_mut(64).enumerate() {
        let t = index as f64 * 64.0 / 48000.0;
        while next < score.notes.len() && score.notes[next].t <= t {
            active = Some(next);
            next += 1;
        }
        if let Some(i) = active {
            let n = &score.notes[i];
            let age = t - n.t;
            let life = n.duration.min(0.26);
            if age < life {
                let u = age / life;
                let breath = (age / 0.012).min(1.0) * exp(-age / 0.14) * n.amplitude;
                voice.set_params(&[
                    (pitch, n.pitch * (1.12 - 0.22 * u)),
                    (pressure, 28.0 * n.pitch * (1.12 - 0.22 * u) / 330.0),
                    (mouth, 0.8 * (1.0 - u) + 0.18 * u),
                ]);
                voice.set_gate(breath);
            } else {
                voice.set_gate(0.0);
            }
        }
        voice.render(block);
    }
    let elapsed = t0.elapsed().as_secs_f64();
    let mut telemetry = voice.new_telemetry();
    voice.telemetry(&mut telemetry);
    if telemetry
        .source
        .iter()
        .any(|(name, v)| *name == "fault" && *v != 0.0)
    {
        return Err("physical source faulted during the score".into());
    }
    let peak = audio.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
    if !audio.iter().all(|v| v.is_finite()) || peak < 1e-4 {
        return Err("render is non-finite or inaudible".into());
    }
    let bytes = u32::try_from(count * 2).map_err(|e| format!("WAV too large: {e}"))?;
    let mut file = File::create(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?;
    let mut data = Vec::with_capacity(count * 2 + 44);
    data.extend_from_slice(b"RIFF");
    data.extend_from_slice(&(bytes + 36).to_le_bytes());
    data.extend_from_slice(b"WAVEfmt ");
    data.extend_from_slice(&16u32.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&48000u32.to_le_bytes());
    data.extend_from_slice(&96000u32.to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());
    data.extend_from_slice(&16u16.to_le_bytes());
    data.extend_from_slice(b"data");
    data.extend_from_slice(&bytes.to_le_bytes());
    for x in audio {
        data.extend_from_slice(
            &((f64::from(x).clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes(),
        );
    }
    file.write_all(&data)
        .map_err(|e| format!("{}: {e}", args[1]))?;
    println!(
        "{}: {} physical bark gestures, {:.3}s rendered in {:.3}s ({:.1}% RT), peak {peak:.4}; source audio inputs: none",
        score.title,
        score.notes.len(),
        duration,
        elapsed,
        elapsed / duration * 100.0
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
