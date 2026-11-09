/** Record the actual lab viewport + its live WebAudio output for both dog methods.
 * No rendered audio file, musician recording, replacement UI or backing track enters capture.
 * Usage: bun tools/capture-dog-demo.ts URL OUTPUT_DIRECTORY
 */
import { chromium, type Page } from "playwright-core";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { createHash } from "node:crypto";
import type { AudioEngine } from "../src/audio/AudioEngine.ts";

const url = process.argv[2];
const root = process.argv[3];
if (!url || !root || process.argv.length !== 4) throw new Error("usage: capture-dog-demo URL OUTPUT_DIRECTORY");
await mkdir(root, { recursive: true });
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const browser = await chromium.launch({ headless: true, args: ["--autoplay-policy=no-user-gesture-required"] });
const results: Record<string, unknown>[] = [];
type CaptureWindow = { __chimeraEngine: AudioEngine; __dogCapture: { recorder: MediaRecorder; done: Promise<Blob>; destination: MediaStreamAudioDestinationNode } };

async function ffmpeg(args: string[], log: string) {
  const p = Bun.spawn(["ffmpeg", "-hide_banner", "-y", ...args], { stdout: "pipe", stderr: "pipe" });
  const [status, stdout, stderr] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
  await writeFile(log, stdout + stderr);
  if (status !== 0) throw new Error(`ffmpeg failed (${status}); see ${log}: ${stderr.slice(-1500)}`);
}

async function take(id: "dog" | "dog-physics") {
  const dir = join(root, id);
  await mkdir(join(dir, "frames"), { recursive: true });
  await writeFile(join(dir, "README.md"), `Actual lab capture created by Codex GPT-6 /root, owner cdjk, 2026-11-09 UTC on cdjks-MacBook-Pro-2.local. Input ${url}, selected ${id}. frames/ are actual Chromium viewport JPEGs; capture.json records timestamps, telemetry and hashes. live-audio.webm is MediaRecorder output from the lab's live post-master analyser, never alternate audio. screen.ffconcat preserves captured timing; encode.log records conversion to ../${id}.mp4. Active requested same-lab video review, next review Nov 10, expiry Nov 16; discard frames/intermediates after QA within 24h. Reproduce with tools/capture-dog-demo.ts at the pinned source revision in the parent inventory. Music/CC0/source rights and complete retention in the parent README. NOT on HF.\n`);
  const page: Page = await browser.newPage({ viewport: { width: 1920, height: 1440 }, deviceScaleFactor: 1 });
  const errors: string[] = [];
  page.on("pageerror", e => errors.push(e.message));
  page.on("console", m => { if (m.type() === "error") errors.push(m.text()); });
  await page.goto(url, { waitUntil: "networkidle" });
  await page.getByRole("region", { name: "Dog song comparison", exact: true }).waitFor();
  await page.getByRole("button", { name: id === "dog" ? "Dog reference" : "Dog physics", exact: true }).click();
  await page.getByRole("button", { name: "Start audio", exact: true }).click();
  await page.waitForFunction(id => (window as unknown as CaptureWindow).__chimeraEngine.latestCreature?.t.id === id, id);
  await page.evaluate(() => document.fonts.ready);
  const cdp = await page.context().newCDPSession(page);
  const frames: { file: string; timestamp: number }[] = [];
  const writes: Promise<unknown>[] = [];
  cdp.on("Page.screencastFrame", event => {
    const number = frames.length;
    const file = `frames/${String(number).padStart(5, "0")}.jpg`;
    const timestamp = event.metadata.timestamp;
    if (typeof timestamp !== "number" || !Number.isFinite(timestamp)) { errors.push("Screencast frame missing timestamp"); writes.push(cdp.send("Page.screencastFrameAck", { sessionId: event.sessionId })); return; }
    frames.push({ file, timestamp });
    writes.push(writeFile(join(dir, file), Buffer.from(event.data, "base64")).then(() => cdp.send("Page.screencastFrameAck", { sessionId: event.sessionId })).catch(e => { errors.push(String(e)); }));
  });
  await cdp.send("Page.startScreencast", { format: "jpeg", quality: 88, maxWidth: 1920, maxHeight: 1440, everyNthFrame: 2 });
  for (let n = 0; n < 20 && !frames.length; n++) await page.waitForTimeout(50);
  if (!frames.length) throw new Error("No actual lab frames captured");
  const audioEpoch = await page.evaluate(async () => {
    const w = window as unknown as CaptureWindow;
    const e = w.__chimeraEngine;
    if (!e.ctx || !e.analyserFast || !MediaRecorder.isTypeSupported("audio/webm;codecs=opus")) throw new Error("Live audio capture unavailable");
    const destination = e.ctx.createMediaStreamDestination();
    e.analyserFast.connect(destination);
    const recorder = new MediaRecorder(destination.stream, { mimeType: "audio/webm;codecs=opus", audioBitsPerSecond: 192000 });
    const chunks: Blob[] = [];
    const done = new Promise<Blob>((resolve, reject) => {
      recorder.ondataavailable = e => { if (e.data.size) chunks.push(e.data); };
      recorder.onerror = () => reject(new Error("MediaRecorder failed"));
      recorder.onstop = () => resolve(new Blob(chunks, { type: recorder.mimeType }));
    });
    w.__dogCapture = { recorder, done, destination };
    return await new Promise<number>(resolve => { recorder.onstart = () => resolve(performance.timeOrigin + performance.now()); recorder.start(250); });
  });
  await page.waitForTimeout(850);
  await page.getByRole("button", { name: "Play song", exact: true }).click();
  await page.waitForFunction(() => (window as unknown as CaptureWindow).__chimeraEngine.latestCreature?.t.call === "score");
  const timing = await page.evaluate(() => (window as unknown as CaptureWindow).__chimeraEngine.lastCreatureScore);
  if (!timing || timing.creature !== id) throw new Error("Wrong live score/source selected");
  const telemetry: unknown[] = [];
  let peak = 0;
  let released = false;
  const deadline = Date.now() + (timing.end - timing.t0 + 3) * 1000;
  while (Date.now() < deadline) {
    await page.waitForTimeout(80);
    const t = await page.evaluate(() => (window as unknown as CaptureWindow).__chimeraEngine.latestCreature?.t);
    if (!t || t.id !== id || t.source.fault) throw new Error("Live source changed or faulted during capture");
    if (id === "dog-physics" && t.call === "score" && t.source.sourceModel !== 1) throw new Error("Physical take left the body-cover model");
    if (id === "dog" && Object.values(t.tubes).some(tube => tube.rms.some(v => v !== 0))) throw new Error("Reference fabricated acoustic pressure");
    peak = Math.max(peak, t.outRms);
    telemetry.push({ clock: t.clock, rms: t.outRms, call: t.call, source: t.source, params: t.params });
    if (t.call !== "score" && t.clock >= timing.end) { released = true; break; }
  }
  if (!released || peak < 0.01) throw new Error(`Incomplete/inaudible live take: release=${released}, peak=${peak}`);
  await page.waitForTimeout(850);
  const audio = await page.evaluate(async () => {
    const w = window as unknown as CaptureWindow;
    const capture = w.__dogCapture;
    capture.recorder.stop();
    const blob = await capture.done;
    w.__chimeraEngine.analyserFast!.disconnect(capture.destination);
    for (const track of capture.destination.stream.getTracks()) track.stop();
    const bytes = new Uint8Array(await blob.arrayBuffer());
    let binary = "";
    for (let i = 0; i < bytes.length; i += 8192) binary += String.fromCharCode(...bytes.subarray(i, i + 8192));
    return { base64: btoa(binary), stopEpoch: performance.timeOrigin + performance.now(), mime: blob.type };
  });
  await cdp.send("Page.stopScreencast");
  await Promise.all(writes);
  await page.close();
  if (errors.length) throw new Error(errors.join("\n"));
  for (let i = 1; i < frames.length; i++) if (frames[i].timestamp <= frames[i - 1].timestamp) throw new Error("Non-monotonic viewport timestamps");
  const raw = Buffer.from(audio.base64, "base64");
  await writeFile(join(dir, "live-audio.webm"), raw);
  const first = frames[0].timestamp;
  const offset = audioEpoch / 1000 - first;
  if (offset < 0 || offset > 3) throw new Error(`Unexpected live audio/video epoch offset: ${offset}`);
  const end = Math.max(frames.at(-1)!.timestamp + 0.08, audio.stopEpoch / 1000);
  let concat = "ffconcat version 1.0\n";
  frames.forEach((f, i) => { concat += `file '${f.file}'\nduration ${((frames[i + 1]?.timestamp ?? end) - f.timestamp).toFixed(6)}\n`; });
  concat += `file '${frames.at(-1)!.file}'\n`;
  await writeFile(join(dir, "screen.ffconcat"), concat);
  await writeFile(join(dir, "capture.json"), JSON.stringify({ source: id, url, frames, audioEpoch, stopEpoch: audio.stopEpoch, offsetSeconds: offset, audioSha256: hash(raw), timing, peakIntervalRms: peak, telemetry, method: "Actual lab screencast + same page live post-master WebAudio capture. No alternate render or audio source.", }, null, 2));
  await ffmpeg(["-safe", "0", "-i", join(dir, "screen.ffconcat"), "-i", join(dir, "live-audio.webm"), "-vf", "fps=30,format=yuv420p", "-af", `adelay=${Math.round(offset * 1000)}:all=1`, "-c:v", "libx264", "-preset", "fast", "-crf", "18", "-c:a", "aac", "-b:a", "192k", "-t", (end - first).toFixed(3), "-movflags", "+faststart", join(root, `${id}.mp4`)], join(dir, "encode.log"));
  results.push({ source: id, frames: frames.length, duration: end - first, offsetSeconds: offset, peakIntervalRms: peak, output: `${id}.mp4`, audioSha256: hash(raw) });
  console.log(JSON.stringify(results.at(-1)));
}

try {
  await take("dog");
  await take("dog-physics");
  await writeFile(join(root, "takes.ffconcat"), "ffconcat version 1.0\nfile 'dog.mp4'\nfile 'dog-physics.mp4'\n");
  await ffmpeg(["-safe", "0", "-i", join(root, "takes.ffconcat"), "-c", "copy", "-movflags", "+faststart", join(root, "dog-comparison.mp4")], join(root, "combine.log"));
  await writeFile(join(root, "capture-summary.json"), JSON.stringify({ method: "Same lab, same requested notes/timing. Recorded reference first, actual physical source second. Live browser audio in each take.", takes: results }, null, 2));
} finally { await browser.close(); }
