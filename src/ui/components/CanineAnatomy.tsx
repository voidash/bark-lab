import { useRef, useState, type PointerEvent, type KeyboardEvent } from "react";
import { engine } from "../../audio/AudioEngine.ts";
import { creatureById } from "../../wasm/engine.ts";
import { useStore } from "../store.ts";
import { previewTelemetry, useLiveCreature } from "../creatureLive.ts";
import { ANATOMY_CONTROLS, anatomyGeometry, dragValues, line, type CaninePart, type Point } from "../canineAnatomy.ts";
import { heat, pressureLevel } from "../colors.ts";

const COLORS: Record<CaninePart, string> = { jaw: "#ffb340", tongue: "#5be381", root: "#5be381", velum: "#4fb6f2", larynx: "#ff73cf" };
const NAMES: Record<CaninePart, string> = { jaw: "Jaw", tongue: "Tongue", root: "Tongue root", velum: "Velum", larynx: "Larynx" };
const HELP: Record<CaninePart, string> = { jaw: "Up/down opens and closes the jaw", tongue: "Up/down changes elevation; left/right changes position", root: "Left/right retracts and advances the tongue root", velum: "Up closes and down opens the nasal route", larynx: "Up/down changes larynx descent" };

export function CanineAnatomy() {
  const def = creatureById("dog-physics");
  const params = useStore(s => s.cparams["dog-physics"]);
  const started = useStore(s => s.started);
  const starting = useStore(s => s.starting);
  const startError = useStore(s => s.startError);
  const hold = useStore(s => s.hold);
  const view = useStore(s => s.view);
  const live = useLiveCreature("dog-physics");
  const svg = useRef<SVGSVGElement>(null);
  const drag = useRef<{ part: CaninePart; at: Point; initial: Record<string, number> } | null>(null);
  const [message, setMessage] = useState("Drag a colored handle. Arrow keys and sliders also work.");
  if (!def) return <p role="alert">Physical dog metadata is unavailable. Reload the lab.</p>;
  const tele = live ?? previewTelemetry(def, params);
  if (!["mouth", "tongueHeight", "tonguePosition", "tongueRoot", "nasal", "larynxDrop"].every(k => Number.isFinite(tele.params[k])) || !tele.tubes.oral?.areas?.length) return <p role="alert">Canine anatomy metadata is out of date. Reload the lab.</p>;
  const values = Object.fromEntries(def.params.map(p => [p.key, params?.[p.key] ?? p.default]));
  const pose = live?.call ? live.params : { ...tele.params, ...values };
  // While running, the lumen follows solved geometry rather than guessing from a
  // control value. Before audio starts, previewTelemetry runs the same model.
  const geo = anatomyGeometry({ ...tele, params: pose });
  const { upper, lower, handles, larynx, jawAngle } = geo;
  const setValues = (next: Record<string, number>) => {
    engine.stopCall();
    for (const [key, value] of Object.entries(next)) useStore.getState().setCParam(key, value);
  };
  const position = (event: PointerEvent): Point | null => {
    const matrix = svg.current?.getScreenCTM();
    if (!matrix) { setMessage("Drawing is not ready. Try again or use the sliders."); return null; }
    const p = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) { setMessage("Drawing coordinates are unavailable. Use the sliders or reload the lab."); return null; }
    return [p.x, p.y];
  };
  const down = (event: PointerEvent<SVGGElement>, part: CaninePart) => {
    if (event.button !== 0) return;
    engine.stopCall();
    const at = position(event);
    if (!at) return;
    event.currentTarget.focus();
    drag.current = { part, at, initial: { ...values } };
    svg.current!.setPointerCapture(event.pointerId);
    event.preventDefault();
    setMessage(`${NAMES[part]} selected. ${HELP[part]}.`);
  };
  const move = (event: PointerEvent<SVGSVGElement>) => {
    if (!drag.current) return;
    const p = position(event), d = drag.current;
    if (!p) { drag.current = null; return; }
    setValues(dragValues(d.part, p[0] - d.at[0], p[1] - d.at[1], d.initial));
  };
  const up = (event: PointerEvent<SVGSVGElement>) => {
    drag.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
  };
  const key = (event: KeyboardEvent, part: CaninePart) => {
    const delta: Record<string, Point> = { ArrowUp: [0, -2], ArrowDown: [0, 2], ArrowLeft: [-4, 0], ArrowRight: [4, 0] };
    const d = delta[event.key];
    if (!d) return;
    event.preventDefault();
    setValues(dragValues(part, d[0], d[1], values));
    setMessage(`${NAMES[part]} adjusted. ${HELP[part]}.`);
  };
  const sustain = async () => {
    await useStore.getState().start();
    if (useStore.getState().started) useStore.getState().setHold(!useStore.getState().hold);
  };
  const reset = () => {
    const keys: string[] = [...ANATOMY_CONTROLS.map(c => c.key), "tractLength", "cavity", "throat"];
    setValues(Object.fromEntries(def.params.filter(p => keys.includes(p.key)).map(p => [p.key, p.default])));
    setMessage("Anatomy controls reset to the model's resting settings.");
  };
  const pressure = (tube: string) => {
    const rms = tele.tubes[tube]?.rms;
    return view.pressure && live && rms?.length ? heat(pressureLevel(Math.max(...rms))) : "#667983";
  };
  const oralPath = `${line(upper)} ${line([...lower].reverse()).replace(/^M/, "L")} Z`;
  const tonguePath = `${line(lower)} L805,410 Q630,465 394,450 Q338,450 ${geo.rootEdge},419 Z`;
  const ly = larynx[1];
  const labels = view.calipers;
  return <div className="canine-anatomy">
    <div className="canine-anatomy-toolbar">
      <span>Sagittal anatomy study</span>
      <div>
        <button className="chip" onClick={() => useStore.getState().playCall("bark")} disabled={starting}>{started ? "Bark now" : starting ? "Starting…" : "Start physical dog"}</button>
        <button className={`chip${hold ? " on" : ""}`} aria-pressed={hold} onClick={() => void sustain()} disabled={starting}>{hold ? "Release breath" : "Sustain"}</button>
        <button className="chip" onClick={reset}>Reset anatomy</button>
      </div>
    </div>
    {startError && <p className="error" role="alert">Audio could not start: {startError}</p>}
    <svg ref={svg} className="canine-anatomy-plate" viewBox="0 0 900 660" role="group" aria-label="Dog vocal tract: controllable sagittal cutaway with tongue, jaw, palate, epiglottis and larynx" onPointerMove={move} onPointerUp={up} onPointerCancel={up}>
      <desc>Reference-informed anatomy illustration. Colored handles adjust the reduced acoustic model. Use arrow keys on a focused handle or use the sliders below.</desc>
      <defs>
        <linearGradient id="canine-tissue" x1="0" y1="0" x2="0.8" y2="1"><stop stopColor="#666d72"/><stop offset="0.5" stopColor="#343b40"/><stop offset="1" stopColor="#202a31"/></linearGradient>
        <linearGradient id="canine-tongue" x1="0" y1="0" x2="0" y2="1"><stop stopColor="#7d8284"/><stop offset="1" stopColor="#414a50"/></linearGradient>
        <radialGradient id="canine-brain"><stop stopColor="#667074"/><stop offset="1" stopColor="#363f46"/></radialGradient>
      </defs>
      <path d="M80 625 Q105 460 131 350 Q128 238 171 151 Q204 57 325 43 Q448 43 499 143 Q541 213 640 244 L822 283 Q855 291 850 328 L826 356 Q783 367 749 373 Q693 441 408 473 Q325 502 288 626 Z" fill="url(#canine-tissue)" stroke="#9aa4a9" strokeWidth="2"/>
      <path d="M175 216 Q166 116 259 77 Q362 39 433 111 Q465 147 471 204 Q371 244 269 230 Z" fill="url(#canine-brain)" stroke="#b7bec1" strokeWidth="10"/>
      <g fill="none" stroke="#7e8b91" strokeWidth="3" opacity=".75">
        <path d="M215 196 Q192 150 242 118 Q270 93 289 116 Q316 69 345 102 Q390 78 410 120 Q448 140 431 180"/>
        <path d="M226 204 Q268 177 278 143 M284 210 Q303 171 330 144 M349 212 Q362 170 393 150"/>
      </g>
      <path d="M461 229 Q568 243 821 294 L824 322 Q637 312 486 305 Q444 309 399 352 L322 411 L289 391 Q338 314 461 279 Z" fill="#070d12" stroke="#697982" strokeWidth="2"/>
      <g fill="none" stroke="#73818a" strokeWidth="2">
        <path d="M518 259 Q563 279 532 292 Q592 264 603 288 Q633 270 657 292 Q680 279 714 302"/>
        <path d="M546 269 Q591 295 616 284 Q644 301 679 292 Q717 304 765 309"/>
        <path d="M493 289 Q519 277 526 301 M587 276 L574 306 M644 287 L636 309 M703 298 L699 313"/>
      </g>
      <path d="M468 316 Q626 315 808 335 L809 348 Q625 330 470 335 Z" fill="#c2c7c5" stroke="#d6dcda" strokeWidth="2"/>
      <path d={tonguePath} fill="url(#canine-tongue)" stroke="#a2aaac" strokeWidth="2"/>
      <g opacity=".35" fill="none" stroke="#a9b0b2" strokeWidth="1.5">
        <path d={`M${geo.rootEdge + 8} 432 Q496 396 667 406 M410 443 Q539 420 764 399 M445 444 Q564 431 716 421`}/>
      </g>
      <path d={oralPath} fill="#03090c" stroke={pressure("oral")} strokeWidth="2.5"/>
      <path d={`M310 ${ly} Q273 437 314 398 Q344 365 390 335 L390 367 Q343 386 338 420 Q318 447 337 ${ly} Z`} fill="#03090c" stroke={pressure("pharynx")} strokeWidth="2.5"/>
      <path d={`M295 ${ly + 12} Q240 541 196 638 L223 638 Q270 553 326 ${ly + 14} Z`} fill="#03090c" stroke={pressure("trachea")} strokeWidth="2.5"/>
      <g fill="none" stroke="#929c9f" strokeWidth="5" opacity=".8">
        {Array.from({length: 7}, (_, i) => { const y = ly + 40 + i * (570 - ly) / 7, x = 308 - (y - ly) * .65; return <path key={i} d={`M${x - 15} ${y} l24 13`}/>; })}
      </g>
      <g transform={`rotate(${jawAngle} 390 395)`}>
        <path d="M363 441 Q573 466 811 386 L821 403 Q663 493 388 471 Z" fill="#adb5b7" stroke="#d2d7d6" strokeWidth="2"/>
        <path d="M778 397 L787 372 L799 391 M742 412 L749 396 L759 407" fill="#d1d6d3"/>
      </g>
      <path d="M775 338 L768 366 L754 336 M803 342 L798 354 L788 340" fill="#dce0db"/>
      <path d={`M474 327 Q423 325 383 345 Q364 351 ${handles.velum[0]} ${handles.velum[1]} Q${handles.velum[0] + 18} ${handles.velum[1] + 8} 401 365 Q430 343 475 337 Z`} fill="#929a9c" stroke="#c1c8c8" strokeWidth="2"/>
      <path d={`M330 ${ly - 11} Q320 ${ly - 64} 362 ${ly - 78} Q371 ${ly - 67} 347 ${ly - 49} L349 ${ly - 10} Z`} fill="#7e9095" stroke="#c5cdce" strokeWidth="2"/>
      <path d={`M283 ${ly - 3} Q309 ${ly - 28} 348 ${ly - 5} L341 ${ly + 24} Q310 ${ly + 39} 286 ${ly + 17} Z`} fill="#555f65" stroke="#bbc3c6" strokeWidth="3"/>
      <path d={`M291 ${ly + 12} Q317 ${ly + 3} 341 ${ly + 15}`} fill="none" stroke="#e6eeed" strokeWidth="3"/>
      <path d={`M286 ${ly + 30} Q313 ${ly + 19} 334 ${ly + 31}`} fill="none" stroke="#a6b4ba" strokeWidth="4"/>
      <path d={`M289 ${ly - 39} L366 ${ly - 34} L391 ${ly - 9}`} fill="none" stroke="#c5c8bd" strokeWidth="5"/>
      <ellipse cx="831" cy="304" rx="11" ry="7" fill="#020506"/>
      {labels && <g className="canine-anatomy-labels">
        <text x="242" y="139">Cranial cavity</text><text x="621" y="243">Nasal cavity</text>
        <text x="621" y="309">Hard palate</text><text x="489" y="417">Tongue</text>
        <text x="359" y="300">Soft palate / velum</text><path d={`M393 306 L${handles.velum[0] + 14} ${handles.velum[1] - 7}`}/>
        <text x="140" y="390">Pharynx</text><path d="M214 387 L299 412"/>
        <text x="403" y={ly - 57}>Epiglottis</text><path d={`M399 ${ly - 54} L359 ${ly - 60}`}/>
        <text x="130" y={ly + 13}>Larynx / vocal folds</text><path d={`M251 ${ly + 17} L283 ${ly + 10}`}/>
        <text x="97" y="568">Trachea</text><text x="636" y={457 + pose.mouth * 42}>Mandible</text>
      </g>}
      <g className="canine-anatomy-handles">
        {(Object.keys(handles) as CaninePart[]).map(part => <g key={part} data-handle={part} role="button" tabIndex={0} aria-label={`${NAMES[part]} handle. ${HELP[part]}`} onPointerDown={e => down(e, part)} onKeyDown={e => key(e, part)} style={{color: COLORS[part]}}>
          <circle cx={handles[part][0]} cy={handles[part][1]} r="22" className="handle-hit"/>
          <circle cx={handles[part][0]} cy={handles[part][1]} r="9" className="handle-ring"/><circle cx={handles[part][0]} cy={handles[part][1]} r="2.5" fill="currentColor"/>
          <title>{NAMES[part]} — {HELP[part]}</title>
        </g>)}
      </g>
      <text x="45" y="646" className="canine-plate-note">Reference-informed illustration · not an MRI segmentation · dimensions are model assumptions</text>
    </svg>
    <p className="canine-handle-help" aria-live="polite">{message}</p>
    <div className="canine-anatomy-controls" aria-label="Canine articulators">
      {ANATOMY_CONTROLS.map(control => <label key={control.key}>
        <span>{control.label}<output>{values[control.key].toFixed(2)}{"unit" in control ? ` ${control.unit}` : ""}</output></span>
        <input aria-label={control.label} type="range" min={control.min} max={control.max} step={control.step} value={values[control.key]} onChange={e => setValues({[control.key]: Number(e.target.value)})}/>
      </label>)}
    </div>
    <p className="canine-anatomy-note">Colored handles change the acoustic model. Gray tissue is illustrative; the oral lumen follows simulated cross-sections. The epiglottis and nasal detail are landmarks, not independently simulated tissues.</p>
  </div>;
}
