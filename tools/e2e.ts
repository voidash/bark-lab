/**
 * Headless-Chromium smoke test: loads the app, starts audio, interacts, collects console errors,
 * measures worklet load, and saves screenshots.
 * Usage: bun tools/e2e.ts [url] [outdir]
 */
import { chromium } from "playwright-core";
import { mkdirSync } from "node:fs";

const url = process.argv[2] ?? "http://127.0.0.1:5318/";
const outDir = process.argv[3] ?? "out/e2e";
mkdirSync(outDir, { recursive: true });

const browser = await chromium.launch({
  headless: true,
  args: ["--autoplay-policy=no-user-gesture-required", "--use-fake-ui-for-media-stream"],
});
const errors: string[] = [];
const logs: string[] = [];
async function run(viewport: { width: number; height: number }, tag: string) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 1 });
  page.on("console", (m) => {
    const t = `${m.type()}: ${m.text()}`;
    if (m.type() === "error") errors.push(`[${tag}] ${t}`);
    else logs.push(`[${tag}] ${t}`);
  });
  page.on("pageerror", (e) => errors.push(`[${tag}] pageerror: ${e.message}`));
  await page.goto(url, { waitUntil: "networkidle" });
  // The dog fork opens on its canine airway. This part of the suite exercises the
  // original human controls explicitly, rather than relying on the landing mode.
  await page.getByRole("radio", { name: "Play", exact: true }).click();
  await page.waitForTimeout(400);
  await page.screenshot({ path: `${outDir}/${tag}-0-idle.png` });
  const startBtn = page.getByRole("button", { name: /breathe in and start/i });
  await startBtn.click();
  await page.waitForTimeout(1800);
  await page.screenshot({ path: `${outDir}/${tag}-1-running.png` });
  // drag the tongue body handle via the canvas: find it by probing (center-ish of viewport)
  const box = await page.locator(".anatomy canvas").boundingBox();
  if (box) {
    // move over likely tongue area and drag up/forward
    const sx = box.x + box.width * 0.47, sy = box.y + box.height * 0.42;
    await page.mouse.move(sx, sy);
    await page.mouse.down();
    await page.mouse.move(sx + 40, sy - 30, { steps: 10 });
    await page.mouse.up();
  }
  // vowel map drag
  const vm = await page.locator(".vowel-map").boundingBox();
  if (vm) {
    await page.mouse.move(vm.x + vm.width * 0.2, vm.y + vm.height * 0.2);
    await page.mouse.down();
    await page.mouse.move(vm.x + vm.width * 0.8, vm.y + vm.height * 0.8, { steps: 20 });
    await page.mouse.up();
  }
  await page.keyboard.down("d");
  await page.waitForTimeout(900);
  await page.screenshot({ path: `${outDir}/${tag}-2-played.png` });
  await page.keyboard.up("d");
  const loadText = await page.locator(".load").textContent().catch(() => null);
  logs.push(`[${tag}] load: ${loadText}`);
  // speak mode
  await page.getByRole("radio", { name: "Speak" }).click();
  await page.waitForTimeout(200);
  await page.getByRole("button", { name: /^Speak$/ }).click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${outDir}/${tag}-3-speak.png`, fullPage: true });
  const trans = await page.locator(".transcription").textContent();
  logs.push(`[${tag}] transcription: ${trans}`);
  // speech breathes by itself: Play's held breath must not drone on once the phrase is over
  const speechRms: number[] = [];
  for (let i = 0; i < 12; i++) {
    await page.waitForTimeout(250);
    speechRms.push(
      await page.evaluate(() => {
        // @ts-expect-error debug handle
        return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
      }),
    );
  }
  logs.push(`[${tag}] speech rms then silence ${speechRms.map((r) => r.toFixed(4)).join(",")}`);
  if (!(Math.max(...speechRms.slice(-3)) < 1e-3)) errors.push(`[${tag}] voice still sounding after the phrase: ${speechRms.slice(-3).join(",")}`);
  await page.locator("button.ipa", { hasText: "ʃ" }).first().click();
  await page.waitForTimeout(700);
  await page.screenshot({ path: `${outDir}/${tag}-4-pad.png` });
  // sing mode: a key sings the marked syllable and moves the cursor; legato; a demo song
  await page.getByRole("radio", { name: "Sing" }).click();
  await page.waitForTimeout(600);
  const rms = () =>
    page.evaluate(() => {
      // @ts-expect-error debug handle
      return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
    });
  const nextSyl = () => page.locator(".syl.next").textContent().catch(() => null);
  const sylBefore = await nextSyl();
  await page.keyboard.down("d");
  await page.waitForTimeout(450);
  const singing = await rms();
  await page.keyboard.down("g"); // legato into the next syllable
  await page.waitForTimeout(350);
  await page.keyboard.up("d");
  await page.waitForTimeout(250);
  await page.keyboard.up("g");
  await page.waitForTimeout(900);
  const after = await rms();
  const sylAfter = await nextSyl();
  logs.push(`[${tag}] sing: next syllable ${sylBefore} -> ${sylAfter}, rms while held ${singing.toFixed(4)}, then silence ${after.toFixed(4)}`);
  if (!(singing > 0.01)) errors.push(`[${tag}] a sung key made no sound (${singing})`);
  if (!(after < 1e-3)) errors.push(`[${tag}] voice still sounding after the sung notes: ${after}`);
  if (sylBefore === sylAfter) errors.push(`[${tag}] the syllable cursor did not move (${sylBefore})`);
  await page.getByRole("button", { name: "Made of air" }).click();
  await page.waitForTimeout(2200);
  const demoRms = await rms();
  const demoNow = await page.locator(".syl.now").textContent().catch(() => null);
  await page.screenshot({ path: `${outDir}/${tag}-5-sing.png`, fullPage: tag !== "desktop" });
  await page.getByRole("button", { name: /^Stop “Made of air”$/ }).click();
  await page.waitForTimeout(700);
  const demoStopped = await rms();
  logs.push(`[${tag}] sing demo: rms ${demoRms.toFixed(4)} on "${demoNow}", after stop ${demoStopped.toFixed(4)}`);
  if (!(demoRms > 0.01) || !demoNow) errors.push(`[${tag}] demo song not singing (rms ${demoRms}, syllable ${demoNow})`);
  if (!(demoStopped < 1e-3)) errors.push(`[${tag}] demo still sounding after stop: ${demoStopped}`);
  // bestiary: every creature, its first call, a key in held-voice mode, and the sustain toggle
  await page.getByRole("radio", { name: "Bestiary" }).click();
  await page.waitForTimeout(300);
  const names = await page.locator(".specimen-name").allTextContents();
  for (const [i, name] of names.entries()) {
    await page.locator(".specimen").nth(i).click();
    await page.waitForTimeout(250);
    await page.locator(".call").first().click();
    await page.waitForTimeout(1300);
    const slug = name.toLowerCase().replace(/[^a-z]+/g, "-");
    if (tag === "desktop" || i === 0) await page.screenshot({ path: `${outDir}/${tag}-5-${slug}.png`, fullPage: tag !== "desktop" });
    const state = await page.evaluate(() => {
      // @ts-expect-error debug handle
      const e = window.__chimeraEngine;
      const t = e?.latestCreature?.t;
      return t ? { id: t.id, rms: t.outRms, call: t.call, load: e.load } : null;
    });
    logs.push(`[${tag}] creature ${name}: ${JSON.stringify(state)}`);
  }
  await page.keyboard.down("g");
  await page.waitForTimeout(600);
  const held = await page.evaluate(() => {
    // @ts-expect-error debug handle
    const t = window.__chimeraEngine?.latestCreature?.t;
    return t ? { gate: t.gate, rms: t.outRms } : null;
  });
  await page.keyboard.up("g");
  logs.push(`[${tag}] held key: ${JSON.stringify(held)}`);
  // beatbox: preset, play, playhead + level from telemetry, a pad, stop
  await page.getByRole("radio", { name: "Beatbox" }).click();
  await page.waitForTimeout(300);
  await page.getByRole("button", { name: "Breakbeat" }).click();
  await page.getByRole("button", { name: "Play beat" }).click();
  const beatSamples: { step: number; rms: number }[] = [];
  for (let i = 0; i < 8; i++) {
    await page.waitForTimeout(250);
    beatSamples.push(
      await page.evaluate(() => {
        // @ts-expect-error debug handle
        const t = window.__chimeraEngine?.latest;
        return { step: t?.beatStep ?? -2, rms: t?.t?.outRms ?? -1 };
      }),
    );
  }
  await page.screenshot({ path: `${outDir}/${tag}-6-beatbox.png`, fullPage: tag !== "desktop" });
  logs.push(`[${tag}] beatbox steps ${beatSamples.map((b) => b.step).join(",")} rms ${beatSamples.map((b) => b.rms.toFixed(3)).join(",")}`);
  await page.locator(".pad").first().dispatchEvent("pointerdown");
  await page.waitForTimeout(300);
  await page.getByRole("button", { name: "Stop" }).click();
  await page.waitForTimeout(300);
  // back to the human voice
  await page.getByRole("radio", { name: "Play" }).click();
  await page.waitForTimeout(600);
  const human = await page.evaluate(() => {
    // @ts-expect-error debug handle
    const e = window.__chimeraEngine;
    return { human: !!e?.latest, creature: !!e?.latestCreature };
  });
  logs.push(`[${tag}] back to play: ${JSON.stringify(human)}`);
  await page.close();
}

// A fresh visit must show the dog and start the dog engine, without a manual mode switch.
{
  const page = await browser.newPage({ viewport: { width: 1440, height: 980 } });
  page.on("pageerror", (e) => errors.push(`[dog-start] pageerror: ${e.message}`));
  await page.goto(url, { waitUntil: "networkidle" });
  if ((await page.getByRole("radio", { name: "Bestiary", exact: true }).getAttribute("aria-checked")) !== "true") errors.push("[dog-start] Bestiary is not selected on a fresh visit");
  if ((await page.locator(".specimen.on .specimen-name").textContent())?.trim() !== "Dog physics") errors.push("[dog-start] physical Dog is not selected on a fresh visit");
  if ((await page.locator(".left .screen-title").textContent()) !== "Dog vocal tract") errors.push("[dog-start] the left screen does not identify the dog vocal tract");
  if ((await page.getByRole("img", { name: /^Dog vocal tract:/ }).count()) !== 1) errors.push("[dog-start] no accessible dog vocal tract drawing");
  if (await page.locator(".body-control").count()) errors.push("[dog-start] human body controls are visible on the dog view");
  if ((await page.getByRole("checkbox", { name: "Sound pressure", exact: true }).count()) !== 1) errors.push("[dog-start] physical voice lacks a pressure toggle");
  await page.screenshot({ path: `${outDir}/dog-start.png` });
  await page.getByRole("button", { name: "Start physical dog", exact: true }).click();
  await page.waitForFunction(() => {
    // @ts-expect-error debug handle
    return window.__chimeraEngine?.latestCreature?.t?.id === "dog-physics";
  });
  const dog = await page.evaluate(() => {
    // @ts-expect-error debug handle
    const t = window.__chimeraEngine.latestCreature.t;
    return { id: t.id, tubes: Object.keys(t.tubes) };
  });
  if (!["trachea", "pharynx", "oral", "nasal"].every((name) => dog.tubes.includes(name))) errors.push(`[dog-start] missing canine airway branches: ${JSON.stringify(dog)}`);
  logs.push(`[dog-start] creature ${JSON.stringify(dog)}`);
  for (const id of ["dog-physics", "dog"]) {
    const physical = id === "dog-physics";
    if (!physical) {
      await page.locator(".specimen").filter({ hasText: "Dog reference" }).click();
      await page.waitForFunction(() => {
        // @ts-expect-error debug handle
        return window.__chimeraEngine?.latestCreature?.t?.id === "dog";
      });
      if (await page.getByRole("checkbox", { name: "Sound pressure", exact: true }).count()) errors.push("[dog-start] recorded voice exposes a pressure toggle");
    }
    for (const name of ["Bark", "Howl", "Yodel"]) {
      await page.getByRole("button", { name, exact: true }).click();
      let peak = 0, pressure = false;
      for (let i = 0; i < 24; i++) {
        await page.waitForTimeout(50);
        const sample = await page.evaluate(() => {
          // @ts-expect-error debug handle
          const t = window.__chimeraEngine.latestCreature.t;
          return { rms: t.outRms, source: t.source, pressure: Object.values(t.tubes).some((tube) => (tube as { rms: number[] }).rms.some((v) => v !== 0)) };
        });
        peak = Math.max(peak, sample.rms);
        pressure ||= sample.pressure;
        if (!physical && sample.pressure) errors.push(`[dog-start] ${name} fabricates airway pressure`);
        if (physical && sample.source.fault !== 0) errors.push(`[dog-start] ${name} mechanical source fault`);
        if (!Number.isFinite(sample.source[physical ? "glottis" : "recording"])) errors.push(`[dog-start] ${id} ${name} lacks valid source telemetry`);
      }
      if (!(peak > (physical ? 0.001 : 0.005))) errors.push(`[dog-start] ${id} ${name} is silent: ${peak}`);
      if (physical && !pressure) errors.push(`[dog-start] ${name} lacks physical airway energy`);
      logs.push(`[dog-start] creature ${id} ${name} peak interval rms ${peak}, pressure ${pressure}`);
    }
  }
  await page.screenshot({ path: `${outDir}/dog-start-running.png` });
  await page.close();
}

// the very first key in Sing mode, before audio has started, still sings once audio is up
{
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => errors.push(`[first-key] pageerror: ${e.message}`));
  await page.goto(url, { waitUntil: "networkidle" });
  await page.getByRole("radio", { name: "Sing" }).click();
  await page.keyboard.down("d");
  await page.waitForTimeout(1200);
  const level = await page.evaluate(() => {
    // @ts-expect-error debug handle
    return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
  });
  await page.keyboard.up("d");
  logs.push(`[first-key] sing before start: rms ${level.toFixed(4)}`);
  if (!(level > 0.01)) errors.push(`[first-key] the first sung key, pressed before audio started, made no sound (${level})`);
  await page.close();
}

await run({ width: 1440, height: 980 }, "desktop");
await run({ width: 390, height: 844 }, "mobile");
await browser.close();
console.log("console errors:", errors.length ? "\n" + errors.join("\n") : "none");
console.log(logs.filter((l) => /load|warn|transcription|creature|held|back to|beatbox|silence|sing/i.test(l)).slice(0, 40).join("\n"));
// a run that collected errors fails, so CI and agents can't read it as a pass
if (errors.length) process.exit(1);
