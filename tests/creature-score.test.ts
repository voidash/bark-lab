import { expect, test } from "bun:test";
import { wasmModule } from "./harness.ts";
import { parseDogSong } from "../src/ui/dogSong.ts";

test("local score metadata rejects malformed input before playback", () => {
  const score = { keys: [{ t: 0, values: { gate: 1 } }, { t: 0.3, values: { gate: 0 } }] };
  const song = { title: "Local test", description: "Two controls", duration: 0.3, voices: { dog: score, "dog-physics": score } };
  expect(parseDogSong(song)).toEqual(song);
  for (const bad of [null, {}, {...song,duration:NaN}, {...song,voices:{dog:score}}, {...song,voices:{...song.voices,dog:{keys:[{t:0,values:{gate:1}},{t:0.3,values:{gate:1}}]}}}]) expect(() => parseDogSong(bad)).toThrow();
});

test("worklet local scores preserve source identity, release and recover from invalid input", async () => {
  type Proc = {port:{onmessage:((e:{data:unknown})=>void)|null};process:(i:unknown,o:Float32Array[][])=>boolean};
  let Ctor: (new(o:unknown)=>Proc)|null=null;
  const posted: {type:string; t?: import("../src/wasm/types.ts").CreatureTelemetry;message?:string}[]=[];
  const g=globalThis as Record<string,unknown>;
  g.sampleRate=48000;
  g.AudioWorkletProcessor=class {port={onmessage:null as unknown,postMessage:(m:{type:string})=>posted.push(m)};};
  g.registerProcessor=(_name:string,ctor:typeof Ctor)=>(Ctor=ctor);
  // @ts-expect-error isolate the processor registration for this test's host
  await import("../src/worklet/processor.ts?local-score");
  const p=new Ctor!({processorOptions:{module:wasmModule}});
  const send=(data:unknown)=>p.port.onmessage!({data});
  const tick=(n:number)=>{for(let i=0;i<n;i++)p.process([],[[new Float32Array(128)]]);};
  for(const id of ["dog","dog-physics"]) {
    send({type:"creature",id});
    posted.length=0;
    const values=id==="dog"?{gate:1,pitch:330,voice:0,"!bark":1}:{gate:1,pitch:330,sourceModel:1};
    send({type:"creatureScore",score:{keys:[{t:0,values},{t:0.4,values:{gate:0}}]}});
    tick(40);
    expect(posted.some(m=>m.type==="scoreStarted")).toBe(true);
    const t=posted.filter(m=>m.type==="ctelemetry").at(-1)!.t!;
    expect(t.call).toBe("score"); expect(t.outRms).toBeGreaterThan(0.001);
    if(id==="dog") {expect(t.source.recording).toBeGreaterThan(0);expect(Object.values(t.tubes).every(t=>t.rms.every(v=>v===0))).toBe(true);}
    else {
      const snapshots=posted.filter(m=>m.type==="ctelemetry").map(m=>m.t!);
      expect(snapshots.every(t=>t.source.sourceModel===1 && t.source.fault===0)).toBe(true);
      // A snapshot can land on glottal closure. Require flow over the interval,
      // rather than incorrectly requiring the aperture to be open at every read.
      expect(Math.max(...snapshots.map(t=>Math.abs(t.source.glottalFlow)))).toBeGreaterThan(0);
    }
    send({type:"creatureScore",score:{keys:[]}}); tick(6);
    expect(posted.some(m=>m.type==="scoreError")).toBe(true);
    expect(posted.filter(m=>m.type==="ctelemetry").at(-1)!.t!.call).toBe("score");
    send({type:"stopCall"}); tick(200);
    const tail=posted.filter(m=>m.type==="ctelemetry").at(-1)!.t!;
    expect(tail.call).toBe(null);expect(tail.outRms).toBeLessThan(1e-5);
    expect(posted.filter(m=>m.type==="error")).toEqual([]);
  }
});
