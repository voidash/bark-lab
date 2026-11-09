import { usePhosphor } from "../usePhosphor.ts";
import { useStore } from "../store.ts";
import { creatureById } from "../../wasm/engine.ts";
import { liveTelemetry, previewTelemetry } from "../creatureLive.ts";
import { V } from "../colors.ts";
import { beam, dot, label, seg } from "../render/vector.ts";

/** Actual cover edges and, for v2, body masses. No invented phase or slowed cycle. */
export function CanineSource() {
  const ref = usePhosphor(({ ctx, w, h }) => {
    const state = useStore.getState();
    const def = creatureById(state.creature);
    if (!def || def.id !== "dog-physics") return;
    const live = liveTelemetry(def.id);
    const source = (live ?? previewTelemetry(def, state.cparams[def.id])).source;
    const bodyCover = source.sourceModel >= 0.5;
    if (!["restArea", "foldLength", "leftUpper", "leftLower", "rightUpper", "rightLower", "lungPressure", "subPressure", "supraPressure", "glottalArea", "glottalFlow", ...(bodyCover ? ["leftBody", "rightBody"] : [])].every(k => Number.isFinite(source[k]))) {
      label(ctx, "MECHANICAL TELEMETRY MISSING: RELOAD THE LAB", 12, 28, { size: 9, color: V.amber });
      return;
    }
    const compact = w < 430;
    const diagramW = compact ? w * 0.53 : w * 0.48;
    const cx = diagramW / 2;
    const upper = 52, lower = h - 47;
    const half = source.restArea / (2 * source.foldLength);
    const outer = Math.min(66, diagramW * 0.32);
    const excursion = Math.max(...[source.leftUpper, source.leftLower, source.rightUpper, source.rightLower].map(x => Math.abs(half + x)), ...(bodyCover ? [Math.abs(source.leftBody), Math.abs(source.rightBody)] : []), 1e-5);
    const scale = Math.min(900, diagramW * 4, outer * (bodyCover ? 0.4 : 0.7) / excursion);
    const bodies = [cx - outer * 0.7 - (source.leftBody ?? 0) * scale, cx + outer * 0.7 + (source.rightBody ?? 0) * scale];
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
        const outside = bodyCover ? bodies[side < 0 ? 0 : 1] : cx + side * outer;
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
    if (bodyCover) {
      for (const x of bodies) { seg(ctx, x, upper - 12, x, lower + 12, V.pink, 5); dot(ctx, x, (upper + lower) / 2, 4, V.pink); }
      label(ctx, "BODY + COVER", cx, 22, { size: 8, color: V.pink, align: "center" });
    }
    label(ctx, bodyCover ? "6 MASSES / MAGNIFIED" : "4 MASSES / MAGNIFIED", cx, h - 9, { size: 8, color: V.blueDim, align: "center" });

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
  return <canvas ref={ref} className="instrument-canvas" role="img" aria-label="Canine larynx: simulated cover and body motion, opening, airflow and boundary pressures" />;
}

export function CanineResearch() {
  return <details className="canine-research">
    <summary>Research and model assumptions</summary>
    <p>Breath drives left/right vocal folds; their opening changes airflow into the throat, mouth and nose. The airway pressure feeds back on the tissue. The body–cover candidate adds two independently moving body masses beneath four cover-edge masses; pink bars show their actual solved motion.</p>
    <ul>
      <li><a href="https://pubmed.ncbi.nlm.nih.gov/10504322/" target="_blank" rel="noreferrer">Riede &amp; Fitch, 1999</a>: canine tract length and non-uniform resonances. Our airway shape is a tunable approximation, not a scan.</li>
      <li><a href="https://pubmed.ncbi.nlm.nih.gov/11681395/" target="_blank" rel="noreferrer">Riede et al., 2001</a>: barks combine periodic vibration and irregular/noisy energy. Adding noise alone is not canine validation.</li>
      <li><a href="https://pmc.ncbi.nlm.nih.gov/articles/PMC8547019/" target="_blank" rel="noreferrer">Jiang et al., 2021/2022</a>: measured canine geometry and tissue properties support a coupled flow–tissue model. We have not reproduced its MRI-based continuum simulation.</li>
      <li><a href="https://www.isca-archive.org/interspeech_2016/moore16_interspeech.pdf" target="_blank" rel="noreferrer">Moore, 2016</a>: real-time mammalian source/filter architecture, including a dog preset; it had no formal perceptual evaluation.</li>
    </ul>
    <p><a href="https://vanat.ahc.umn.edu/carnLabs/Lab23/Img23-11.html" target="_blank" rel="noreferrer">University of Minnesota canine anatomy</a> informs the original cutaway's tissue relationships. No scan was segmented; tongue and larynx controls use bounded acoustic approximations.</p>
    <p>Legacy uses generic mammalian tissue constants and an effective 14 mm fold length. The body–cover candidate uses 12.5 mm length, 3.5 mm height, density and body/cover elastic moduli from the canine study. Lateral depths (0.5/2.5 mm), damping, the lumped reduction and call gestures are assumptions. Neither is a continuum/MRI reconstruction or a validated dog voice. Snapshots come from the solver; displacement is magnified.</p>
    <p>Based on <a href="https://github.com/jamesondh/chimera-vocal-synth" target="_blank" rel="noreferrer">CHIMERA</a>. Its human stroboscope and 3D anatomy cannot stand in for a measured canine model.</p>
  </details>;
}
