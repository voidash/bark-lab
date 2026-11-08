/** Small, fixed acoustic comparison for the physical dog experiment; no perceptual verdict. */
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { FFT, hann } from "./lib/fft.ts";
import { readWav } from "./lib/wav.ts";

const out = resolve(process.argv[2] ?? "../research/dog-physics");
const rate = 24000;
const edges = [80, 160, 250, 400, 630, 1000, 1600, 2500, 4000, 6300, 10000];
function pcm(path: string): Float64Array {
  const bytes = readFileSync(path);
  if (!bytes.length || bytes.length % 2) throw new Error(`Invalid PCM16 source: ${path}`);
  return Float64Array.from({ length: bytes.length / 2 }, (_, i) => bytes.readInt16LE(i * 2) / 32768);
}
function wav(path: string): Float64Array {
  const w = readWav(path);
  if (w.channels.length !== 1 || !w.channels[0].length) throw new Error(`Expected nonempty mono WAV: ${path}`);
  const x = w.channels[0], speed = w.sampleRate / rate;
  return Float64Array.from({ length: Math.floor(x.length / speed) }, (_, i) => {
    const at = i * speed, k = Math.floor(at), f = at - k;
    return x[k] * (1 - f) + x[Math.min(x.length - 1, k + 1)] * f;
  });
}
function features(x: Float64Array) {
  if (!x.length || x.some((v) => !Number.isFinite(v))) throw new Error("Empty/non-finite comparison audio");
  const hop = 240;
  const envelope: number[] = [];
  for (let i = 0; i + hop <= x.length; i += hop) envelope.push(Math.sqrt(x.slice(i, i + hop).reduce((a, b) => a + b*b, 0) / hop));
  const maximum = Math.max(...envelope);
  if (!(maximum > 1e-8)) throw new Error("Cannot compare silence");
  const active = envelope.map((v, i) => v > maximum * 0.1 ? i : -1).filter((i) => i >= 0);
  const fft = new FFT(1024), window = hann(1024);
  const bands = edges.slice(1).map(() => 0);
  let frames = 0;
  for (let at = 0; at + 1024 <= x.length; at += hop) {
    if (envelope[Math.floor((at + 512) / hop)] <= maximum * 0.1) continue;
    const re = new Float64Array(1024), im = new Float64Array(1024);
    for (let i = 0; i < 1024; i++) re[i] = x[at+i] * window[i];
    fft.transform(re, im);
    for (let b = 0; b < bands.length; b++) for (let k = Math.ceil(edges[b]*1024/rate); k < Math.floor(edges[b+1]*1024/rate); k++) bands[b] += re[k]**2 + im[k]**2;
    frames++;
  }
  if (!frames) throw new Error("No active spectral frames");
  const total = bands.reduce((a,b) => a+b, 0);
  return { activeSeconds: (active.at(-1)! - active[0] + 1) * hop / rate,
    attackSeconds: (envelope.indexOf(maximum) - active[0]) * hop / rate,
    bandDb: bands.map((v) => 10 * Math.log10(Math.max(1e-8, v / total))),
    envelope: envelope.map((v) => v / maximum) };
}
const references = {
  bark: features(pcm("crates/chimera/src/creatures/data/dog-bark.pcm")),
  howl: features(pcm("crates/chimera/src/creatures/data/dog-howl.pcm")),
};
const result: Record<string, unknown> = { method: "10 ms RMS; active above -20 dB relative peak; active 1024-point Hann frames; normalized energy in fixed 80–10000 Hz bands; spectral distance is RMS dB difference. Acoustic diagnostic only, not a canine-likeness score.", edges, references };
for (const [name, folder, prefix] of [
  ["rejected", "../research/dog-prototype", "dog"],
  ["physical", out, "dog-physics"],
] as const) {
  const clips: Record<string, unknown> = {};
  for (const call of ["bark", "howl"] as const) {
    const f = features(wav(resolve(folder, `${prefix}-${call}.wav`)));
    const r = references[call];
    clips[call] = { ...f, spectralDistanceDb: Math.sqrt(f.bandDb.reduce((sum, v, i) => sum + (v-r.bandDb[i])**2, 0) / f.bandDb.length) };
  }
  result[name] = clips;
}
writeFileSync(resolve(out, "acoustic-comparison.json"), JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify(result, (_key, value) => Array.isArray(value) && value.length > 20 ? `[${value.length} values]` : value, 2));
