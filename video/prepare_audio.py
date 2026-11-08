"""Prepare a clearly sample-based dog performance and frame-synchronous animation data.

No download, training, vocal-tract claim or source-song waveform in the dog bus.
Optional reference vocals supply pitch/dynamics only; optional backing is mixed separately.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import subprocess

import librosa
import numpy as np
import soundfile as sf

SR = 44100
FPS = 30
REPO = Path(__file__).resolve().parent.parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path, stereo=False):
    data, rate = sf.read(path, always_2d=True, dtype="float32")
    if not len(data) or data.shape[1] not in [1, 2] or not np.isfinite(data).all():
        raise ValueError(f"Expected finite nonempty mono/stereo audio: {path}")
    if rate != SR:
        data = librosa.resample(data.T, orig_sr=rate, target_sr=SR).T
    return np.repeat(data, 2, axis=1) if stereo and data.shape[1] == 1 else data if stereo else data.mean(axis=1)


def bank():
    root = REPO/"crates/chimera/src/creatures/data"
    manifest = json.loads((root/"manifest.json").read_text())
    result = {}
    for row in manifest["files"]:
        path = root/row["file"]
        if path.parent != root or digest(path) != row["sha256"] or row["format"] != "signed PCM16 little endian, no header":
            raise ValueError(f"Source manifest/hash mismatch: {path}")
        data = np.frombuffer(path.read_bytes(), dtype="<i2").astype(np.float32)/32768
        if len(data) != row["frames"]:
            raise ValueError(f"Source frame count mismatch: {path}")
        data = librosa.resample(data, orig_sr=row["sampleRate"], target_sr=SR)
        data, _ = librosa.effects.trim(data, top_db=32, frame_length=256, hop_length=64)
        result[path.stem] = data
    if set(result) != {"dog-bark", "dog-howl"}:
        raise ValueError("Expected the verified bark and howl bank")
    return result


def synthetic_backing(seconds):
    """Original simple groove for the public source-only example."""
    t = np.arange(round(seconds*SR))/SR
    y = np.zeros_like(t)
    rng = np.random.default_rng(401)
    for beat in np.arange(0, seconds, 0.5):
        i = round(beat*SR)
        local = np.arange(min(round(0.3*SR),len(y)-i))/SR
        y[i:i+len(local)] += 0.3*np.sin(2*np.pi*(48*local+9*(1-np.exp(-local*35))))*np.exp(-local*18)
        if round(beat*2) % 2:
            y[i:i+len(local)] += rng.uniform(-1,1,len(local))*np.exp(-local*38)*0.12
    return np.column_stack([y,y]).astype(np.float32)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--score",type=Path,required=True)
    p.add_argument("--outdir",type=Path,required=True)
    p.add_argument("--backing",type=Path)
    p.add_argument("--vocals",type=Path)
    p.add_argument("--deep-bark",type=Path)
    p.add_argument("--duration",type=float,default=22)
    p.add_argument("--lead-offset",type=float,default=2.66)
    p.add_argument("--title",default="Bark Lab warm-up")
    p.add_argument("--tag",action="store_true",help="Add an arranged bark response at the ending")
    a = p.parse_args()
    if not math.isfinite(a.duration) or not 5 <= a.duration <= 120 or not 0 <= a.lead_offset < a.duration:
        raise ValueError("Require duration 5–120 seconds and a valid lead offset")
    score = json.loads(a.score.read_text())
    notes = score.get("notes")
    if not isinstance(notes,list) or not 1 <= len(notes) <= 1000:
        raise ValueError("Score must contain 1–1000 note objects")
    last = -1.0
    for n in notes:
        if not isinstance(n,dict) or not all(isinstance(n.get(k),(int,float)) and math.isfinite(n[k]) for k in ["t","duration","pitch","amplitude"]):
            raise ValueError("Each note needs finite t/duration/pitch/amplitude")
        if n["t"] < 0 or n["t"] < last or not 0.04 <= n["duration"] <= 2 or not 50 <= n["pitch"] <= 2000 or not 0 <= n["amplitude"] <= 2:
            raise ValueError("Invalid or unsorted note")
        last = n["t"]
    count = round(a.duration*SR)
    backing = load(a.backing,True) if a.backing else synthetic_backing(a.duration)
    if len(backing) < count-100:
        raise ValueError("Backing shorter than requested composition")
    backing = backing[:count]
    if len(backing) < count:
        backing = np.pad(backing,((0,count-len(backing)),(0,0)))
    backing *= 0.10/max(1e-8,float(np.sqrt(np.mean(backing**2))))
    samples = bank()
    deep = load(a.deep_bark) if a.deep_bark else samples["dog-bark"]
    deep,_ = librosa.effects.trim(deep,top_db=30,frame_length=256,hop_length=64)
    sources = [samples["dog-bark"],deep,samples["dog-bark"],samples["dog-howl"]]
    f0 = times = probability = vocal = None
    if a.vocals:
        vocal = load(a.vocals)
        mono = librosa.resample(vocal,orig_sr=SR,target_sr=22050)
        f0,_,probability = librosa.pyin(mono,fmin=75,fmax=650,sr=22050,frame_length=2048,hop_length=512,resolution=0.2)
        times = librosa.times_like(f0,sr=22050,hop_length=512)
    events = []
    for i,n in enumerate(notes):
        t = float(n["t"])+a.lead_offset
        if t >= a.duration-0.5:
            raise ValueError("Score event lies outside the composition")
        measured = None
        if f0 is not None:
            selected = f0[(times>=t)&(times<t+n["duration"])&(probability>0.25)]
            selected = selected[np.isfinite(selected)]
            if len(selected):
                measured = float(np.median(selected))
        pitch = measured if measured is not None else float(n["pitch"])
        while pitch < 220:
            pitch *= 2
        while pitch > 620:
            pitch *= 0.5
        # Artistic roles: the lead handles the opening and middle calls; the crew answers.
        dog = 0 if n["t"]<2.1 or 4.5<=n["t"]<6.3 or 8<=n["t"]<11 else 1+i%2
        events.append({"t":t,"duration":min(0.44,max(0.12,float(n["duration"])+0.04)),"dog":dog,
                       "pitch":pitch,"amplitude":min(1.2,max(0.5,float(n["amplitude"]))),
                       "pitchFromStem":measured is not None,"kind":"bark"})
    centre = float(np.median([n["pitch"] for n in events]))
    # One howl in an actual internal gap, leaving room before the next bark.
    gaps = [(events[i+1]["t"]-(e["t"]+e["duration"]),e["t"]+e["duration"]) for i,e in enumerate(events[:-1])]
    if gaps:
        length,start = max(gaps)
        if length > 1.1:
            events.append({"t":start+0.12,"duration":min(1.35,length-0.3),"dog":3,"pitch":centre,"amplitude":0.8,"pitchFromStem":False,"kind":"howl"})
    if a.tag:
        for i in range(4):
            events.append({"t":a.duration-3.6+i*0.45,"duration":0.27,"dog":[1,2,1,0][i],"pitch":centre*2**([0,3,0,-2][i]/12),"amplitude":1.1,"pitchFromStem":False,"kind":"bark"})
    events.sort(key=lambda e:e["t"])
    voices = np.zeros((4,count),dtype=np.float32)
    for e in events:
        target = round(e["duration"]*SR)
        steps = 12*math.log2(e["pitch"]/centre)+[0,-2.5,3.0,0][e["dog"]]
        shifted = librosa.effects.pitch_shift(sources[e["dog"]],sr=SR,n_steps=steps,n_fft=1024,hop_length=128)
        grain = librosa.effects.time_stretch(shifted,rate=len(shifted)/target,n_fft=1024,hop_length=128)
        grain = librosa.util.fix_length(grain,size=target)
        fade = min(round(0.012*SR),target//4)
        grain[:fade] *= np.linspace(0,1,fade)
        grain[-fade:] *= np.linspace(1,0,fade)
        rms = float(np.sqrt(np.mean(grain**2)))
        if not rms > 1e-7 or not np.isfinite(grain).all():
            raise ValueError("Processed dog grain is silent/non-finite")
        grain *= 0.145*e["amplitude"]/rms
        start = round(e["t"]*SR)
        take = min(target,count-start)
        voices[e["dog"],start:start+take] += grain[:take]
    dogs = np.zeros((count,2),dtype=np.float32)
    for i,pan in enumerate([-0.30,-0.65,0.65,0.3]):
        dogs[:,0] += voices[i]*math.sqrt((1-pan)/2)
        dogs[:,1] += voices[i]*math.sqrt((1+pan)/2)
    # Duck accompaniment under the actual dog voice bus, not the original singer.
    envelope = np.sqrt(np.convolve(np.mean(dogs**2,axis=1),np.ones(441)/441,mode="same"))
    mix = dogs+backing*(0.85-0.3*np.clip(envelope/0.16,0,1))[:,None]
    fade = round(0.9*SR)
    mix[:round(0.12*SR)] *= np.linspace(0,1,round(0.12*SR))[:,None]
    mix[-fade:] *= np.linspace(1,0,fade)[:,None]
    mix *= 0.87/max(0.87,float(np.max(np.abs(mix))))
    a.outdir.mkdir(parents=True,exist_ok=True)
    raw = a.outdir/"master-pre.wav"
    sf.write(raw,mix,SR,subtype="FLOAT")
    sf.write(a.outdir/"dogs-only.wav",dogs*min(1,0.9/max(1e-8,float(np.max(np.abs(dogs))))),SR,subtype="PCM_16")
    master = a.outdir/"master.wav"
    subprocess.run(["ffmpeg","-v","error","-y","-i",str(raw),"-af","loudnorm=I=-14:TP=-1.5:LRA=9","-ar","48000","-c:a","pcm_s16le",str(master)],check=True)
    levels,spectrum = [],[]
    mono = mix.mean(axis=1)
    reference = max(1e-6,float(np.percentile(np.abs(voices),99.5)))
    for frame in range(math.ceil(a.duration*FPS)):
        at = round(frame*SR/FPS)
        end = min(count,round((frame+1)*SR/FPS))
        levels.append([round(min(1,float(np.sqrt(np.mean(voices[i,at:end]**2)))/reference*2.5),4) for i in range(4)])
        part = np.zeros(2048)
        take = min(2048,count-at)
        part[:take] = mono[at:at+take]
        fft = np.abs(np.fft.rfft(part*np.hanning(2048)))
        bins = np.geomspace(40,14000,33)*2048/SR
        spectrum.append([round(float(np.clip((20*np.log10(max(1e-7,float(np.max(fft[int(bins[b]):max(int(bins[b])+1,int(bins[b+1]))]))))+35)/65,0,1)),4) for b in range(32)])
    inputs = {"score":{"path":str(a.score.resolve()),"sha256":digest(a.score)}}
    for key in ["backing","vocals","deep_bark"]:
        path = getattr(a,key)
        if path:
            inputs[key] = {"path":str(path.resolve()),"sha256":digest(path)}
    props = {"title":a.title,"durationSeconds":a.duration,"audio":"master.wav","repo":"github.com/voidash/bark-lab","levels":levels,"spectrum":spectrum,"events":events}
    (a.outdir/"props.json").write_text(json.dumps(props,allow_nan=False,separators=(",",":"))+"\n")
    manifest = {"creator":"Codex GPT-6 session /root","createdUtc":datetime.now(timezone.utc).isoformat(),"method":"CC0 sample pitch/time processing; relative pitch intervals; source-separated backing if supplied. Not a physical vocal-tract performance. Vocal stem is analysis-only; possible residual leakage in separated backing.","inputs":inputs,"duration":a.duration,"events":len(events),"stemPitchEvents":sum(e["pitchFromStem"] for e in events),"masterSha256":digest(master),"dogBusSha256":digest(a.outdir/"dogs-only.wav")}
    (a.outdir/"manifest.json").write_text(json.dumps(manifest,indent=2)+"\n")
    print(json.dumps(manifest))


if __name__=="__main__":
    main()
