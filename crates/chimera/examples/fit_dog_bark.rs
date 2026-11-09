//! Offline acoustic fitting of a physical bark. Audio is an evaluation target only.
//! No reference sample is passed to CreatureVoice, the source or the airway.
use chimera::analysis::fft::{Fft, hann};
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureVoice, creature_by_id};
use chimera::math::{exp, log10, pow};
use chimera::util::optimize::{NelderMeadOpts, nelder_mead};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const SR: usize = 48000;
const FRAMES: usize = SR / 2;
const KNOBS: [(&str, f64, f64); 15] = [
    ("pitch", 150.0, 750.0),
    ("rough", 0.0, 0.85),
    ("pressure", 8.0, 48.0),
    ("damping", 0.7, 1.8),
    ("throat", 0.18, 1.8),
    ("tractLength", 17.0, 22.4),
    ("cavity", 0.4, 2.0),
    ("aspiration", 0.05, 2.0),
    ("gap", 0.4, 1.6),
    ("mouthPeak", 0.2, 1.0),
    ("mouthEnd", 0.05, 0.4),
    ("onset", 0.0, 0.04),
    ("attack", 0.004, 0.04),
    ("decay", 0.03, 0.16),
    ("pitchFall", 0.0, 0.45),
];

#[derive(Clone, Serialize, Deserialize)]
struct Bark {
    parameters: BTreeMap<String, f64>,
}

impl Bark {
    fn from_unit(x: &[f64]) -> Self {
        Self {
            parameters: KNOBS
                .iter()
                .zip(x)
                .map(|(&(k, lo, hi), &v)| (k.to_owned(), lo + v * (hi - lo)))
                .collect(),
        }
    }
    fn value(&self, key: &str) -> Result<f64, String> {
        self.parameters
            .get(key)
            .copied()
            .ok_or_else(|| format!("missing bark control {key}"))
    }
    fn validate(&self) -> Result<(), String> {
        if self.parameters.len() != KNOBS.len() {
            return Err("bark configuration must contain exactly the documented controls".into());
        }
        for (k, lo, hi) in KNOBS {
            let value = self.value(k)?;
            if !value.is_finite() || !(lo..=hi).contains(&value) {
                return Err(format!("{k} must be finite and within {lo}–{hi}"));
            }
        }
        Ok(())
    }
}

fn render(bark: &Bark) -> Result<Vec<f32>, String> {
    bark.validate()?;
    let def = creature_by_id("dog-physics").ok_or("physical dog missing")?;
    let parameter = |k| {
        def.param_index(k)
            .ok_or_else(|| format!("missing physical control {k}"))
    };
    let initial: Vec<_> = KNOBS[..9]
        .iter()
        .map(|(k, _, _)| Ok((parameter(k)?, bark.value(k)?)))
        .collect::<Result<_, String>>()?;
    let mut initial = initial;
    initial.extend([
        (parameter("tone")?, 0.0),
        (parameter("nasal")?, 0.0),
        (parameter("mouth")?, bark.value("mouthEnd")?),
    ]);
    let mut voice = CreatureVoice::new(def, SR as f64, &initial);
    let pitch = parameter("pitch")?;
    let mouth = parameter("mouth")?;
    let fundamental = bark.value("pitch")?;
    let start = bark.value("onset")?;
    let attack = bark.value("attack")?;
    let decay = bark.value("decay")?;
    let fall = bark.value("pitchFall")?;
    let open = bark.value("mouthPeak")?;
    let end = bark.value("mouthEnd")?;
    let mut audio = vec![0.0; FRAMES];
    // A pressure impulse, opening/closing jaw and tissue-tension fall. No imposed
    // audio-rate pulse: all audible periodic/irregular motion is air-driven tissue.
    for (block, out) in audio.chunks_mut(64).enumerate() {
        let t = block as f64 * 64.0 / SR as f64 - start;
        let gate = if t < 0.0 || t > 0.36 {
            0.0
        } else {
            (1.0 - exp(-t / attack)) * exp(-t / decay) * 1.7
        };
        let progress = (t.max(0.0) / 0.24).min(1.0);
        voice.set_gate(gate);
        voice.set_params(&[
            (pitch, fundamental * (1.15 - fall * progress)),
            (mouth, end + (open - end) * (gate / 0.9).clamp(0.0, 1.0)),
        ]);
        voice.render(out);
    }
    let mut telemetry = voice.new_telemetry();
    voice.telemetry(&mut telemetry);
    if telemetry
        .source
        .iter()
        .any(|(k, v)| *k == "fault" && *v != 0.0)
    {
        return Err("latched mechanical source fault".into());
    }
    if !audio.iter().all(|x| x.is_finite()) {
        return Err("non-finite physical output".into());
    }
    if rms(&audio) < 1e-5 {
        return Err("physical output is silent".into());
    }
    Ok(audio)
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

struct Features {
    spectrum: Vec<f64>,
    envelope: Vec<f64>,
    periodicity: Vec<f64>,
}

fn features(audio: &[f32]) -> Features {
    let samples: Vec<_> = audio.iter().step_by(2).map(|&x| f64::from(x)).collect();
    let mut envelope: Vec<_> = samples
        .chunks(240)
        .map(|chunk| (chunk.iter().map(|x| x * x).sum::<f64>() / chunk.len() as f64).sqrt())
        .collect();
    let peak = envelope.iter().copied().fold(1e-12, f64::max);
    for e in &mut envelope {
        *e /= peak;
    }
    let mut spectrum = Vec::new();
    // Coarse spectral energy over three time regions, at two resolutions. Do not
    // optimize waveform phase or reuse an extracted excitation/residual signal.
    for n in [512, 1024] {
        let fft = Fft::new(n);
        let window = hann(n);
        for (begin, end) in [(0, 2400), (2400, 4800), (4800, 9600)] {
            let mut bands = vec![0.0; 24];
            for at in (begin..end - n).step_by(240) {
                let mut re: Vec<_> = (0..n).map(|i| samples[at + i] * window[i]).collect();
                let mut im = vec![0.0; n];
                fft.transform(&mut re, &mut im);
                for (b, energy) in bands.iter_mut().enumerate() {
                    let lo =
                        (80.0 * pow(125.0, b as f64 / 24.0) * n as f64 / 24000.0).ceil() as usize;
                    let hi = (80.0 * pow(125.0, (b + 1) as f64 / 24.0) * n as f64 / 24000.0).ceil()
                        as usize;
                    for k in lo..hi.max(lo + 1) {
                        *energy += re[k] * re[k] + im[k] * im[k];
                    }
                }
            }
            let total = bands.iter().sum::<f64>().max(1e-24);
            spectrum.extend(bands.iter().map(|&v| 10.0 * log10((v / total).max(1e-5))));
        }
    }
    let periodicity = [960, 1680, 2400, 3120]
        .iter()
        .map(|&at| {
            yin_pitch(
                &samples[at..at + 1024],
                24000.0,
                YinOpts {
                    min_f0: Some(100.0),
                    max_f0: Some(1200.0),
                    ..Default::default()
                },
            )
            .aperiodicity
        })
        .collect();
    Features {
        spectrum,
        envelope,
        periodicity,
    }
}

fn loss(a: &Features, b: &Features) -> (f64, f64, f64, f64) {
    let spectral = a
        .spectrum
        .iter()
        .zip(&b.spectrum)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        / a.spectrum.len() as f64;
    let envelope = a
        .envelope
        .iter()
        .zip(&b.envelope)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        / a.envelope.len() as f64;
    let periodicity = a
        .periodicity
        .iter()
        .zip(&b.periodicity)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        / a.periodicity.len() as f64;
    // A spectral-only pass selected an overly periodic source. The second pass
    // explicitly retains time-varying irregularity instead of treating a better
    // coarse spectrum as a convincing bark. These are engineering weights, not
    // a perceptual dog-likeness metric or evidence for deterministic chaos.
    (
        spectral + 150.0 * envelope + 1000.0 * periodicity,
        spectral.sqrt(),
        envelope.sqrt(),
        periodicity.sqrt(),
    )
}

fn write_wav(path: &Path, audio: &[f32]) -> Result<(), String> {
    let peak = audio.iter().map(|x| x.abs()).fold(1e-8f32, f32::max);
    let gain = 0.85 / f64::from(peak);
    let bytes = (audio.len() * 2) as u32;
    let mut data = Vec::with_capacity(audio.len() * 2 + 44);
    data.extend_from_slice(b"RIFF");
    data.extend_from_slice(&(bytes + 36).to_le_bytes());
    data.extend_from_slice(b"WAVEfmt ");
    data.extend_from_slice(&16u32.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&(SR as u32).to_le_bytes());
    data.extend_from_slice(&(SR as u32 * 2).to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());
    data.extend_from_slice(&16u16.to_le_bytes());
    data.extend_from_slice(b"data");
    data.extend_from_slice(&bytes.to_le_bytes());
    for &x in audio {
        let value = (f64::from(x) * gain).clamp(-1.0, 1.0) * 32767.0;
        data.extend_from_slice(&(value.round() as i16).to_le_bytes());
    }
    fs::write(path, data).map_err(|e| format!("{}: {e}", path.display()))
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 3 && args[0] == "--render" {
        let input = fs::read_to_string(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?;
        let bark: Bark = serde_json::from_str(&input).map_err(|e| format!("{}: {e}", args[1]))?;
        let audio = render(&bark)?;
        let root = Path::new(&args[2]);
        fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
        write_wav(&root.join("candidate.wav"), &audio)?;
        println!("Rendered physical bark from parameter JSON; no reference audio read");
        return Ok(());
    }
    if args.len() != 1 {
        return Err(
            "usage: fit_dog_bark output-directory | --render candidate.json output-directory"
                .into(),
        );
    }
    let root = Path::new(&args[0]);
    fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let bytes = include_bytes!("../src/creatures/data/dog-bark.pcm");
    if bytes.len() != 18672 {
        return Err("unexpected bundled PCM length".into());
    }
    let mut reference = vec![0.0; FRAMES];
    for (i, sample) in bytes.chunks_exact(2).enumerate() {
        let value = f32::from(i16::from_le_bytes([sample[0], sample[1]])) / 32768.0;
        reference[i * 2] = value;
        reference[i * 2 + 1] = value;
    }
    let target = features(&reference);
    let baseline_values = [
        330.0, 0.85, 28.0, 0.7, 0.4, 17.0, 1.0, 1.0, 1.0, 0.85, 0.12, 0.0, 0.012, 0.08, 0.3,
    ];
    let baseline: Vec<_> = KNOBS
        .iter()
        .zip(baseline_values)
        .map(|((_, lo, hi), v)| (v - lo) / (hi - lo))
        .collect();
    let mut failed = BTreeMap::<String, usize>::new();
    let mut evaluations = 0;
    let mut evaluate = |x: &[f64]| {
        evaluations += 1;
        match render(&Bark::from_unit(x)) {
            Ok(audio) => loss(&features(&audio), &target).0,
            Err(error) => {
                *failed.entry(error).or_default() += 1;
                1e8
            }
        }
    };
    let baseline_loss = evaluate(&baseline);
    let mut seeds = vec![(baseline_loss, baseline.clone())];
    let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
    for i in 1..=64 {
        let x: Vec<_> = primes
            .iter()
            .map(|&base| {
                let (mut index, mut f, mut x) = (i, 1.0, 0.0);
                while index > 0 {
                    f /= base as f64;
                    x += f * (index % base) as f64;
                    index /= base;
                }
                x
            })
            .collect();
        seeds.push((evaluate(&x), x));
    }
    seeds.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut best = seeds[0].clone();
    for (seed, (value, x)) in seeds.iter().take(3).enumerate() {
        eprintln!("start {seed}: objective {value:.4}");
        let result = nelder_mead(
            &mut evaluate,
            x,
            NelderMeadOpts {
                step: Some(vec![0.07; KNOBS.len()]),
                lo: Some(vec![0.0; KNOBS.len()]),
                hi: Some(vec![1.0; KNOBS.len()]),
                max_iter: Some(180),
                tol: Some(1e-5),
            },
        );
        eprintln!(
            "finish {seed}: objective {:.4}, {} iterations",
            result.fx, result.iters
        );
        if result.fx < best.0 {
            best = (result.fx, result.x);
        }
    }
    let bark = Bark::from_unit(&best.1);
    let audio = render(&bark)?;
    let before_audio = render(&Bark::from_unit(&baseline))?;
    let before = loss(&features(&before_audio), &target);
    let after = loss(&features(&audio), &target);
    write_wav(&root.join("reference.wav"), &reference)?;
    write_wav(&root.join("before.wav"), &before_audio)?;
    write_wav(&root.join("candidate.wav"), &audio)?;
    fs::write(
        root.join("candidate.json"),
        serde_json::to_string_pretty(&bark).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("candidate.json: {e}"))?;
    let report = json!({"method":"Physical-only source and airway; reference PCM enters loss evaluation only. Acoustic fitting to one CC0 bark is not anatomical inversion or perceptual validation.",
        "evaluations":evaluations,"rejectedCandidates":failed,"before":{"objective":before.0,"spectralRmsDb":before.1,"envelopeRms":before.2,"periodicityRms":before.3},
        "after":{"objective":after.0,"spectralRmsDb":after.1,"envelopeRms":after.2,"periodicityRms":after.3},"bark":bark});
    fs::write(
        root.join("fit.json"),
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("fit.json: {e}"))?;
    println!("{report}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
