import { Audio } from "@remotion/media";
import { AbsoluteFill, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from "remotion";

export type BarkEvent = { t: number; duration: number; dog: number; kind: string };
export type BandProps = {
  title: string;
  durationSeconds: number;
  audio: string | null;
  repo: string;
  levels: number[][];
  spectrum: number[][];
  events: BarkEvent[];
};

const INK = "#182622", PAPER = "#f3ecd9", GREEN = "#cbf258";
const DOGS = [
  { name: "RUFUS", role: "THE LEAD", coat: "#ca783d", dark: "#8a482e", accent: "#ed7d56", type: "floppy" },
  { name: "OTTO", role: "LOW END", coat: "#725747", dark: "#3c302b", accent: "#ae9dde", type: "long" },
  { name: "PIP", role: "BIG OPINIONS", coat: "#f6f2dc", dark: "#8d9d85", accent: "#b5d5aa", type: "pointy" },
  { name: "LUNA", role: "THE HOWL", coat: "#526b75", dark: "#253e49", accent: "#98c8df", type: "husky" },
];
const clamp = (v: number) => Math.max(0,Math.min(1,v));
const mono = "'Courier New', monospace";

function Dog({ index, level, frame }: { index: number; level: number; frame: number }) {
  const dog = DOGS[index];
  const open = Math.pow(clamp(level),0.65);
  const jaw = 8+open*67;
  const blink = (frame+index*31)%143 < 5;
  const ear = Math.sin(frame*0.15+index)*3+open*5;
  const pointed = dog.type === "pointy" || dog.type === "husky";
  return <svg viewBox="0 0 500 520" width="100%" height="100%" style={{overflow:"visible"}}>
    <g stroke={INK} strokeWidth="7" strokeLinejoin="round">
      <g transform={`rotate(${-ear},142,160)`}>
        {pointed ? <path d="M108 205 L105 27 Q151 42 207 136 Z" fill={dog.dark}/> : <path d="M151 123 Q61 94 49 208 Q40 305 102 347 Q151 323 164 223 Z" fill={dog.dark}/>}
        {pointed && <path d="M127 154 L128 68 L174 137 Z" fill="#dcac99" strokeWidth="0"/>}
      </g>
      <g transform={`rotate(${ear},358,160)`}>
        {pointed ? <path d="M292 136 Q349 42 395 27 L392 205 Z" fill={dog.dark}/> : <path d="M349 123 Q439 94 451 208 Q460 305 398 347 Q349 323 336 223 Z" fill={dog.dark}/>}
        {pointed && <path d="M326 137 L372 68 L373 154 Z" fill="#dcac99" strokeWidth="0"/>}
      </g>
      <path d="M158 374 L131 497 L371 497 L343 374" fill={dog.coat}/>
      <path d="M125 195 Q116 104 210 92 Q250 74 291 93 Q386 105 377 202 L370 305 Q356 398 250 420 Q144 398 129 305 Z" fill={dog.coat}/>
      {dog.type === "husky" && <path d="M143 174 Q169 132 223 181 L250 242 L278 181 Q332 132 357 174 L349 294 Q324 365 250 390 Q176 365 151 294 Z" fill={PAPER} strokeWidth="0"/>}
      {dog.type === "pointy" && <path d="M155 136 Q181 113 218 129 L202 211 Q164 233 143 199 Z" fill={dog.dark} strokeWidth="0"/>}
      <g transform={`translate(0,${-open*5})`}>
        {blink ? <><path d="M166 211 Q185 200 207 211" fill="none"/><path d="M292 211 Q314 200 334 211" fill="none"/></> : <>
          <ellipse cx="187" cy="208" rx="24" ry="30" fill="#fffdf0"/>
          <ellipse cx="313" cy="208" rx="24" ry="30" fill="#fffdf0"/>
          <ellipse cx={190+Math.sin(frame*0.026+index)*4} cy="212" rx="10" ry="16" fill={dog.type === "husky" ? "#7cb5d2" : INK} strokeWidth="0"/>
          <ellipse cx={310+Math.sin(frame*0.026+index)*4} cy="212" rx="10" ry="16" fill={INK} strokeWidth="0"/>
          <circle cx="194" cy="205" r="4" fill="white" strokeWidth="0"/><circle cx="315" cy="205" r="4" fill="white" strokeWidth="0"/>
        </>}
        <path d={`M164 ${166-open*8} Q185 ${154-open*12} 211 ${168-open*6}`} fill="none" strokeWidth="9"/>
        <path d={`M289 ${168-open*6} Q315 ${154-open*12} 336 ${166-open*8}`} fill="none" strokeWidth="9"/>
      </g>
      <path d={`M188 307 Q250 296 312 307 L302 ${326+jaw} Q250 ${369+jaw} 198 ${326+jaw} Z`} fill={INK}/>
      <ellipse cx="250" cy={337+jaw*0.65} rx={27+open*10} ry={8+open*20} fill="#e28d86" strokeWidth="0"/>
      <path d={`M250 ${338+jaw*0.65} L250 ${350+jaw*0.65}`} stroke="#a95757" strokeWidth="3"/>
      <ellipse cx="210" cy="286" rx="51" ry="39" fill={PAPER}/>
      <ellipse cx="290" cy="286" rx="51" ry="39" fill={PAPER}/>
      <path d="M220 259 Q250 246 280 259 Q280 281 250 289 Q220 281 220 259" fill={INK}/>
      <path d="M250 287 L250 305" fill="none"/>
      {[187,202,298,313].map((x,i)=><circle key={x} cx={x} cy={289+(i%2)*10} r="3.2" fill={INK} strokeWidth="0"/>)}
      <path d="M155 414 Q250 446 345 414 L339 451 Q250 480 161 451 Z" fill={dog.accent}/>
      <circle cx="250" cy="465" r="24" fill={GREEN}/>
      <text x="250" y="475" textAnchor="middle" fontFamily="Arial" fontWeight="900" fontSize="27" fill={INK} strokeWidth="0">{dog.name[0]}</text>
      {index===0 && <g><path d="M91 197 Q89 40 250 34 Q411 40 409 197" fill="none" stroke={INK} strokeWidth="22"/><rect x="64" y="179" width="45" height="88" rx="18" fill={GREEN}/><rect x="391" y="179" width="45" height="88" rx="18" fill={GREEN}/></g>}
      {index===1 && <g fill="none" strokeWidth="9"><rect x="147" y="176" width="81" height="57" rx="15"/><rect x="272" y="176" width="81" height="57" rx="15"/><path d="M228 193 Q250 181 272 193"/></g>}
    </g>
  </svg>;
}

export function BarkBand(props: BandProps) {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const t = frame/fps;
  const row = props.levels[Math.min(frame,props.levels.length-1)] ?? [0,0,0,0];
  const bands = props.spectrum[Math.min(frame,props.spectrum.length-1)] ?? Array(32).fill(0);
  const bass = bands.slice(0,5).reduce((a:number,b:number)=>a+b,0)/5;
  const closing = interpolate(frame,[durationInFrames-61,durationInFrames-37],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"});
  const entrance = spring({frame,fps,config:{damping:22,stiffness:150}});
  const events = props.events.filter(e=>e.t<=t);
  return <AbsoluteFill style={{backgroundColor:PAPER,color:INK,fontFamily:"Arial, Helvetica, sans-serif",overflow:"hidden"}}>
    {props.audio && <Audio src={staticFile(props.audio)}/>}
    <svg width="1920" height="1080" style={{position:"absolute",inset:0}}>
      <defs><pattern id="paper" width="11" height="13" patternUnits="userSpaceOnUse"><circle cx="2" cy="3" r="0.65" fill={INK} opacity=".08"/></pattern></defs>
      <rect width="1920" height="1080" fill="url(#paper)"/>
      <path d="M-60 405 Q960 220 1990 405" stroke={INK} strokeWidth="1.3" opacity=".16" fill="none"/>
      <ellipse cx="960" cy="902" rx="990" ry="128" fill={INK}/>
      <ellipse cx="960" cy="910" rx="860" ry="84" fill="#233831"/>
    </svg>
    <div style={{position:"absolute",left:72,top:47,fontSize:34,fontWeight:900,letterSpacing:-2}}>BARK<span style={{color:"#75912b"}}>//</span>LAB</div>
    <div style={{position:"absolute",right:72,top:48,display:"flex",alignItems:"center",gap:13,fontFamily:mono,fontSize:19,letterSpacing:1}}><div style={{width:12,height:12,backgroundColor:"#de7255",borderRadius:12}}/>SAMPLE-BASED PERFORMANCE</div>
    <div style={{position:"absolute",left:70,top:122,width:1320,fontSize:112,fontWeight:900,lineHeight:.93,letterSpacing:-6,textTransform:"uppercase",transform:`translateY(${(1-entrance)*32}px)`,opacity:entrance}}>{props.title}</div>
    <div style={{position:"absolute",left:76,top:355,fontFamily:mono,fontSize:22,letterSpacing:3}}>THE CANINE ENSEMBLE <span style={{opacity:.4}}> / </span> TAKE 001</div>
    <div style={{position:"absolute",right:93,top:171,backgroundColor:GREEN,border:`3px solid ${INK}`,borderRadius:"50%",width:177,height:177,transform:`rotate(${8+bass*2}deg)`,display:"flex",flexDirection:"column",alignItems:"center",justifyContent:"center"}}><div style={{fontSize:64,fontWeight:900,lineHeight:1}}>04</div><div style={{fontFamily:mono,fontSize:18,textAlign:"center",lineHeight:1.1}}>DOGS.<br/>ONE GROOVE.</div></div>
    {DOGS.map((dog,i)=>{
      const level = clamp(row[i] ?? 0);
      const pop = spring({frame:Math.max(0,frame-i*5),fps,config:{damping:20,stiffness:160}});
      const bob = 7*Math.sin(frame*.10+i)*(.2+bass)+level*17;
      const tilt = Math.sin(frame*.07+i)*2.1+level*(i%2 ? -3:3);
      const active = props.events.filter(e=>e.dog===i && t>=e.t && t<e.t+Math.min(.4,e.duration)).at(-1);
      return <div key={dog.name} style={{position:"absolute",left:78+i*443,top:402,width:430,height:541,transform:`translateY(${(1-pop)*540-bob}px) rotate(${tilt}deg)`,transformOrigin:"50% 85%"}}>
        <div style={{position:"absolute",left:18,top:71,width:395,height:369,borderRadius:"50%",backgroundColor:dog.accent,opacity:.62,transform:`scale(${1+level*.08})`}}/>
        <div style={{position:"absolute",inset:"0 0 50px 0"}}><Dog index={i} level={level} frame={frame}/></div>
        {active && <div style={{position:"absolute",right:-8,top:45,padding:"8px 17px",background:GREEN,border:`3px solid ${INK}`,fontSize:32,fontWeight:900,letterSpacing:-1,transform:`rotate(-9deg) scale(${.9+level*.18})`}}>{active.kind==="howl" ? "AWOOO" : "WOOF!"}</div>}
        <div style={{position:"absolute",left:0,right:0,bottom:23,textAlign:"center",color:PAPER,fontWeight:900,fontSize:30,letterSpacing:2}}>{dog.name}<span style={{marginLeft:14,fontFamily:mono,fontWeight:400,fontSize:15,color:dog.accent,letterSpacing:1}}>{dog.role}</span></div>
      </div>;
    })}
    <div style={{position:"absolute",left:78,right:78,bottom:78,height:49,display:"flex",alignItems:"flex-end",gap:9}}>{bands.map((v:number,i:number)=><div key={i} style={{flex:1,height:6+v*39,backgroundColor:i<7 ? GREEN : i<20 ? "#acc2ab":"#78a393",opacity:.8}}/>)}</div>
    <div style={{position:"absolute",left:77,right:77,bottom:52,height:3,backgroundColor:"#bac4b8"}}><div style={{width:`${frame/durationInFrames*100}%`,height:3,backgroundColor:INK}}/></div>
    <div style={{position:"absolute",left:78,bottom:18,fontFamily:mono,fontSize:17,letterSpacing:1.4}}>REAL BARKS. PITCH / TIME / PLAY.</div>
    <div style={{position:"absolute",right:78,bottom:18,fontFamily:mono,fontSize:17}}>{String(events.length).padStart(2,"0")} CALLS <span style={{margin:"0 20px",opacity:.35}}>/</span>{String(Math.floor(t)).padStart(2,"0")}:{String(Math.floor(frame%fps/fps*100)).padStart(2,"0")}</div>
    {closing>0 && <AbsoluteFill style={{backgroundColor:`rgba(243,236,217,${closing*.97})`,opacity:closing,display:"flex",alignItems:"center",justifyContent:"center",flexDirection:"column"}}>
      <div style={{fontFamily:mono,fontSize:25,letterSpacing:5,marginBottom:32}}>THE DOGS HAVE ENTERED THE MIX.</div>
      <div style={{fontWeight:900,fontSize:176,lineHeight:1,letterSpacing:-12,transform:`scale(${.94+closing*.06})`}}>BARK<span style={{color:"#86a32f"}}>//</span>LAB</div>
      <div style={{marginTop:47,fontFamily:mono,fontSize:32,backgroundColor:GREEN,padding:"18px 28px"}}>{props.repo}</div>
      <div style={{marginTop:35,fontSize:20,opacity:.7}}>Sample-based demo · built on CHIMERA</div>
    </AbsoluteFill>}
  </AbsoluteFill>;
}
