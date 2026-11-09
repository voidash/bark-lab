import type { CreatureScore } from "../worklet/protocol.ts";

export interface DogSong {
  title: string;
  description: string;
  duration: number;
  voices: Record<"dog" | "dog-physics", CreatureScore>;
}

const record = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

/** Local JSON is untrusted input. Rust validates parameter/event names and bounds
 * against the selected creature before replacing its current performance. */
export function parseDogSong(value: unknown): DogSong {
  if (!record(value) || typeof value.title !== "string" || !value.title.trim() || value.title.length > 120 || typeof value.description !== "string" || value.description.length > 1000 || typeof value.duration !== "number" || !Number.isFinite(value.duration) || value.duration <= 0 || value.duration > 120 || !record(value.voices)) throw new Error("Invalid dog song metadata");
  for (const id of ["dog", "dog-physics"]) {
    const score = value.voices[id];
    if (!record(score) || !Array.isArray(score.keys) || score.keys.length < 2 || score.keys.length > 2000) throw new Error(`Missing/invalid ${id} score`);
    let previous = -1;
    for (const key of score.keys) {
      if (!record(key) || typeof key.t !== "number" || !Number.isFinite(key.t) || key.t < previous || key.t < 0 || key.t > value.duration || !record(key.values) || !Object.keys(key.values).length || Object.keys(key.values).length > 64 || !Object.values(key.values).every(v => typeof v === "number" && Number.isFinite(v)) || (key.curve !== undefined && !["lin", "exp", "step"].includes(String(key.curve)))) throw new Error(`Invalid ${id} gesture`);
      previous = key.t;
    }
    if (score.keys[0].t !== 0 || score.keys.at(-1).values.gate !== 0 || score.keys.at(-1).t !== value.duration) throw new Error(`${id} score must start at zero and release at its duration`);
  }
  return value as unknown as DogSong;
}
