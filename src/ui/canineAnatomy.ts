import type { CreatureTelemetry } from "../wasm/types.ts";

export type CaninePart = "jaw" | "tongue" | "root" | "velum" | "larynx";
export type Point = [number, number];
export const ANATOMY_CONTROLS = [
  { key: "mouth", label: "Jaw opening", min: 0, max: 1, step: 0.01 },
  { key: "tongueHeight", label: "Tongue elevation", min: 0, max: 1, step: 0.01 },
  { key: "tonguePosition", label: "Tongue position", min: 0.25, max: 0.80, step: 0.01 },
  { key: "tongueRoot", label: "Tongue root", min: 0, max: 1, step: 0.01 },
  { key: "nasal", label: "Nasal opening / velum", min: 0, max: 1, step: 0.01 },
  { key: "larynxDrop", label: "Larynx descent", min: 0, max: 2, step: 0.02, unit: "cm" },
] as const;

export function line(points: Point[]): string {
  return points.map(([x, y], i) => `${i ? "L" : "M"}${x.toFixed(2)},${y.toFixed(2)}`).join(" ");
}

export function anatomyGeometry(t: CreatureTelemetry) {
  const p = t.params;
  const oral = t.tubes.oral?.areas;
  if (!oral?.length || !Array.from(oral).every(Number.isFinite)) throw new Error("Canine anatomy requires finite oral cross-sections");
  // Display coordinates are not scan coordinates. A fixed out-of-plane depth is
  // assumed to turn effective cross-sectional area into a sagittal gap. The same
  // engine areas draw both this lumen and the unrolled acoustic tube view.
  const roof = (u: number) => 335 - 12 * Math.sin(Math.PI * u);
  const upper: Point[] = Array.from(oral, (_, i) => { const u = i / Math.max(1, oral.length - 1); return [390 + 420 * u, roof(u)]; });
  const lower: Point[] = upper.map(([x, y], i) => [x, y + Math.max(2, oral[i] * 12)]);
  const at = Math.round(p.tonguePosition * (oral.length - 1));
  const larynx: Point = [310, 470 + 35 * p.larynxDrop];
  const handles: Record<CaninePart, Point> = {
    jaw: [760, 405 + 74 * p.mouth], tongue: lower[at], root: [425 - 34 * p.tongueRoot, 438],
    velum: [354, 360 + p.nasal * 62], larynx,
  };
  return { upper, lower, larynx, handles, rootEdge: 355 - 34 * p.tongueRoot, jawAngle: 15 * p.mouth };
}

export function dragValues(part: CaninePart, dx: number, dy: number, initial: Record<string, number>): Record<string, number> {
  const clamp = (x: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, x));
  switch (part) {
    case "jaw": return { mouth: clamp(initial.mouth + dy / 100, 0, 1) };
    case "tongue": return { tongueHeight: clamp(initial.tongueHeight - dy / 35, 0, 1), tonguePosition: clamp(initial.tonguePosition + dx / 420, 0.25, 0.80) };
    case "root": return { tongueRoot: clamp(initial.tongueRoot - dx / 50, 0, 1) };
    case "velum": return { nasal: clamp(initial.nasal + dy / 62, 0, 1) };
    case "larynx": return { larynxDrop: clamp(initial.larynxDrop + dy / 35, 0, 2) };
  }
}
