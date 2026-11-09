import { usePhosphor } from "../usePhosphor.ts";
import { useStore } from "../store.ts";
import { creatureById } from "../../wasm/engine.ts";
import { liveTelemetry, previewTelemetry } from "../creatureLive.ts";
import { V } from "../colors.ts";
import { beam, dot, label, seg } from "../render/vector.ts";

/** Four actual mechanical degrees of freedom. No invented phase or slowed cycle. */
export function CanineSource() {
  const ref = usePhosphor(({ ctx, w, h }) => {
    const state = useStore.getState();
    const def = creatureById(state.creature);
    if (!def || def.id !== "dog-physics") return;
    const live = liveTelemetry(def.id);
    const source = (live ?? previewTelemetry(def, state.cparams[def.id])).source;
    if (!["restArea", "foldLength", "leftUpper", "leftLower", "rightUpper", "rightLower", "lungPressure", "subPressure", "supraPressure", "glottalArea", "glottalFlow"].every(k => Number.isFinite(source[k]))) {
      label(ctx, "MECHANICAL TELEMETRY MISSING: RELOAD THE LAB", 12, 28, { size: 9, color: V.amber });
      return;
    }
    const compact = w < 430;
    const diagramW = compact ? w * 0.53 : w * 0.48;
    const cx = diagramW / 2;
    const upper = 52, lower = h - 47;
    const half = source.restArea / (2 * source.foldLength);
    const outer = Math.min(66, diagramW * 0.32);
    const excursion = Math.max(...[source.leftUpper, source.leftLower, source.rightUpper, source.rightLower].map(x => Math.abs(half + x)), 1e-5);
    const scale = Math.min(900, diagramW * 4, outer * 0.7 / excursion);
    const left = [cx - (half + source.leftUpper) * scale, cx - (half + source.leftLower) * scale];
    const right = [cx + (half + source.rightUpper) * scale, cx + (half + source.rightLower) * scale];
    label(ctx, "LEFT", cx - outer * 0.7, 18, { size: 9, color: V.blueMid, align: "center" });
    label(ctx, "RIGHT", cx + outer * 0.7, 18, { size: 9, color: V.blueMid, align: "center" });
    seg(ctx, cx, 28, cx, h - 25, V.blueDim, 1, { dash: [3, 4] });
    for (let row = 0; row < 2; row++) {
      const y = row === 0 ? upper : lower;
      const color = row === 0 ? V.white : V.amber;
      const depth = row === 0 ? 16 : 25;
      for (const [edge, side] of [[left[row], -1], [right[row], 1]]) {
        const outside = cx + side * outer;
        const shape = new Path2D();
        shape.rect(Math.min(outside, edge), y - depth / 2, Math.abs(edge - outside), depth);
        beam(ctx, shape, V.blueMid, 1.1);
        seg(ctx, edge, y - depth / 2, edge, y + depth / 2, color, 2, { hot: true });
        dot(ctx, (edge + outside) / 2, y, 3, color);
      }
      label(ctx, row === 0 ? "UPPER" : "LOWER", 6, y + 3, { size: 8, color: V.blueMid });
    }
    seg(ctx, left[0], upper, left[1], lower, V.white, 1);
    seg(ctx, right[0], upper, right[1], lower, V.white, 1);
    label(ctx, "POSITION EXAGGERATED", cx, h - 9, { size: 8, color: V.blueDim, align: "center" });

    const x = diagramW + 8, rightX = w - 12;
    const rows = [
      ["LUNG", source.lungPressure, "cmH₂O", 1],
      ["BELOW FOLDS", source.subPressure, "cmH₂O", 1],
      ["ABOVE FOLDS", source.supraPressure, "cmH₂O", 1],
      ["OPENING", source.glottalArea * 100, "mm²", 2],
      ["FLOW", source.glottalFlow, "cm³/s", 0],
    ] as const;
    rows.forEach(([name, value, unit, digits], i) => {
      const y = 27 + i * ((h - 47) / 5);
      label(ctx, name, x, y - 7, { size: 10, color: V.blue });
      label(ctx, `${value.toFixed(digits)} ${unit}`, rightX, y + 6, { size: compact ? 12 : 14, color: V.white, align: "right" });
    });
    label(ctx, live ? "LIVE SNAPSHOT" : "START AUDIO TO INSPECT", rightX, h - 9, { size: 8, color: V.blueDim, align: "right" });
  });
  return <canvas ref={ref} className="instrument-canvas" role="img" aria-label="Canine larynx: four simulated tissue masses, opening, airflow and boundary pressures" />;
}

export function CanineResearch() {
  return <details className="canine-research">
    <summary>Research and model assumptions</summary>
    <p>Breath drives left/right vocal folds; their opening changes airflow into the throat, mouth and nose. The airway pressure feeds back on the tissue.</p>
    <ul>
      <li><a href="https://pubmed.ncbi.nlm.nih.gov/10504322/" target="_blank" rel="noreferrer">Riede &amp; Fitch, 1999</a>: canine tract length and non-uniform resonances. Our airway shape is a tunable approximation, not a scan.</li>
      <li><a href="https://pubmed.ncbi.nlm.nih.gov/11681395/" target="_blank" rel="noreferrer">Riede et al., 2001</a>: barks combine periodic vibration and irregular/noisy energy. Adding noise alone is not canine validation.</li>
      <li><a href="https://pmc.ncbi.nlm.nih.gov/articles/PMC8547019/" target="_blank" rel="noreferrer">Jiang et al., 2021/2022</a>: measured canine geometry and tissue properties support a coupled flow–tissue model. We have not reproduced its MRI-based continuum simulation.</li>
      <li><a href="https://www.isca-archive.org/interspeech_2016/moore16_interspeech.pdf" target="_blank" rel="noreferrer">Moore, 2016</a>: real-time mammalian source/filter architecture, including a dog preset; it had no formal perceptual evaluation.</li>
    </ul>
    <p>Current limits: generic mammalian tissue constants with an effective 14 mm fold length, a schematic head, and hand-designed call gestures. The four-mass source is physical; convincing dog identity is still unvalidated. The snapshots above come from the solver and are not a slow-motion phonation cycle. Tissue displacement is magnified to remain visible.</p>
    <p>Based on <a href="https://github.com/jamesondh/chimera-vocal-synth" target="_blank" rel="noreferrer">CHIMERA</a>. Its human stroboscope and 3D anatomy cannot stand in for a measured canine model.</p>
  </details>;
}
