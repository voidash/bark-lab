import { useEffect, useState } from "react";
import { engine } from "../../audio/AudioEngine.ts";
import { useStore } from "../store.ts";
import { useLiveCreature } from "../creatureLive.ts";
import { parseDogSong, type DogSong } from "../dogSong.ts";

/** Local scores stay outside Git. Both methods play in this lab's live engine. */
export function DogSongDemo() {
  const id = new URLSearchParams(window.location.search).get("song");
  const creature = useStore(s => s.creature);
  const setCreature = useStore(s => s.setCreature);
  const live = useLiveCreature(creature);
  const [song, setSong] = useState<DogSong | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  useEffect(() => {
    if (!id) return;
    if (!/^[a-z0-9-]{1,64}$/.test(id)) { setError("Invalid local song name"); return; }
    const controller = new AbortController();
    void (async () => {
      try {
        const response = await fetch(`/shows/${id}/score.json`, { signal: controller.signal });
        if (!response.ok) throw new Error(`Song could not load (HTTP ${response.status})`);
        const text = await response.text();
        if (text.length > 1024 * 1024) throw new Error("Song exceeds 1 MiB");
        setSong(parseDogSong(JSON.parse(text))); setError(null);
      } catch (e) {
        if (controller.signal.aborted) return; // normal unmount cancels its pending fetch
        setError(String(e)); console.error("dog song:", e);
      }
    })();
    return () => controller.abort();
  }, [id]);
  if (!id || (creature !== "dog" && creature !== "dog-physics")) return null;
  const playing = live?.call === "score";
  const timing = engine.lastCreatureScore;
  const elapsed = playing && timing?.creature === creature ? Math.max(0, live.clock - timing.t0) : 0;
  const play = async () => {
    if (!song) return;
    setStarting(true); setError(null);
    try {
      await useStore.getState().start();
      if (!useStore.getState().started) throw new Error(useStore.getState().startError ?? "Audio did not start");
      useStore.getState().setHold(false);
      engine.stopCall(); engine.gate(0);
      engine.playCreatureScore(song.voices[creature]);
    } catch (e) { setError(String(e)); console.error("dog song playback:", e); }
    finally { setStarting(false); }
  };
  return <section className="dog-song-demo" aria-label="Dog song comparison">
    <h3>{song?.title ?? "Loading local song…"}</h3>
    <div className="dog-song-actions">
      <button className="chip" aria-pressed={creature === "dog"} onClick={() => { useStore.getState().setHold(false); setCreature("dog"); }}>Dog reference</button>
      <button className="chip" aria-pressed={creature === "dog-physics"} onClick={() => { useStore.getState().setHold(false); setCreature("dog-physics"); }}>Dog physics</button>
      <button className="chip" disabled={!song || starting} onClick={() => playing ? engine.stopCall() : void play()}>{playing ? "Stop song" : starting ? "Starting…" : "Play song"}</button>
    </div>
    <p className="dog-song-method">{creature === "dog" ? "REFERENCE · Recorded dog bark + howl, transposed. The airway drawing is illustrative; no tract simulation generates this voice." : "PHYSICS · Air-driven six-mass body–cover tissue + acoustic tract. Actual output, including pitch locking and timbre limitations. No recorded excitation."}</p>
    <progress aria-label="Song progress" value={elapsed} max={song?.duration ?? 1} />
    <p className="dim">{playing ? `${elapsed.toFixed(1)} / ${song?.duration.toFixed(1)} s · live audio clock` : song?.description ?? "Same requested notes and timing for both voices."}</p>
    {(error || engine.scoreError) && <p role="alert" className="error">{error ?? engine.scoreError}</p>}
  </section>;
}
