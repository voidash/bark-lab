"""Compare the same arrangement's short-term repeatability; not a perceptual grunge score."""
import json
import math
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2] / "research/dog-physics"


def main():
    result = {"method":"24 kHz ffmpeg resampling; 1024-sample frames, 512-sample YIN difference window, lags 20–239; minimum CMND trough. Windows at 40, 70, 100, 130 ms within each of seven notes. Median short-term aperiodicity, not a perceptual grunge score: pitch glides and transients also affect it.","clips":{}}
    for name in ["v0","recorded","clean","physical"]:
        path = ROOT / f"arrangement-{name}.wav"
        raw = subprocess.run(["ffmpeg","-v","error","-i",str(path),"-ar","24000","-ac","1","-f","f64le","pipe:1"],capture_output=True,check=True).stdout
        x = [v[0] for v in struct.iter_unpack("<d",raw)]
        rows = []
        for note in range(7):
            values = []
            for offset in [0.04,0.07,0.1,0.13]:
                at = round((note*0.5+offset)*24000)
                f = x[at:at+1024]
                if len(f)!=1024 or sum(v*v for v in f)/1024 < 1e-8:
                    continue
                differences = [0.0]+[sum((f[i]-f[i+tau])**2 for i in range(512)) for tau in range(1,240)]
                cmnd = [1.0];total = 0.0
                for tau in range(1,240):
                    total += differences[tau]
                    cmnd.append(differences[tau]*tau/(total+1e-20))
                troughs = [cmnd[tau] for tau in range(20,239) if cmnd[tau]<cmnd[tau-1] and cmnd[tau]<cmnd[tau+1]]
                if troughs:
                    values.append(min(troughs))
            if not values:
                raise ValueError(f"No valid analysis windows: {name}, note {note}")
            values.sort(); rows.append(values[len(values)//2])
        ordered = sorted(rows)
        result["clips"][name] = {"perNoteMedianAperiodicity":rows,"median":ordered[len(ordered)//2]}
    (ROOT/"arrangement-analysis.json").write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result,indent=2))


if __name__=="__main__":
    main()
