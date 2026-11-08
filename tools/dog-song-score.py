"""Estimate bark gesture timing from the user's song reference. No source audio in output.

Use the isolated faster-whisper environment documented in research/dog-physics/README.md.
Only event times, amplitudes and estimated pitches are saved, never a lyric transcript.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess

import numpy as np
from faster_whisper import WhisperModel
from huggingface_hub import HfApi


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("reference", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    root = args.output.resolve().parent
    info = HfApi().model_info("Systran/faster-whisper-tiny.en")
    model = WhisperModel("Systran/faster-whisper-tiny.en", revision=info.sha,
                         device="cpu", compute_type="int8", cpu_threads=4,
                         download_root=str(root / "analysis-model"))
    raw = subprocess.run(["ffmpeg", "-v", "error", "-t", "55", "-i", str(args.reference),
                          "-ar", "16000", "-ac", "1", "-f", "f32le", "pipe:1"],
                         capture_output=True, check=True).stdout
    audio = np.frombuffer(raw, dtype="<f4")
    if not len(audio) or not np.isfinite(audio).all():
        raise ValueError("Invalid reference decode")
    segments, _ = model.transcribe(audio, language="en", beam_size=5,
                                  word_timestamps=True, vad_filter=False,
                                  condition_on_previous_text=False)
    words = [w for segment in segments for w in (segment.words or [])]
    normalized = [re.sub("[^a-z]", "", w.word.lower()) for w in words]
    pattern = ["who", "let", "the", "dogs", "out"]
    matches = [i for i in range(len(words)-4) if normalized[i:i+5] == pattern]
    if not matches:
        raise ValueError("ASR did not locate the named hook; no guessed score was written")
    start = words[matches[0]].start
    # Keep a compact opening hook performance, not an unreviewed full-song transcription.
    stop = min(55.0, start + 15.0)
    pitch_raw = subprocess.run(["ffmpeg", "-v", "error", "-t", "55", "-i", str(args.reference),
        "-af", "pan=mono|c0=0.5*c0+0.5*c1,highpass=f=90,lowpass=f=1000",
        "-ar", "8000", "-f", "f32le", "pipe:1"], capture_output=True, check=True).stdout
    x = np.frombuffer(pitch_raw, dtype="<f4").astype(np.float64)
    notes = []
    for w in words:
        if w.start < start or w.start >= stop or w.end <= w.start:
            continue
        estimates = []
        for t in np.arange(w.start, max(w.start+0.02, w.end-0.06), 0.02):
            frame = x[round(t*8000):round(t*8000)+640]
            if len(frame) < 640:
                continue
            difference = np.array([0.0] + [np.sum((frame[:480]-frame[tau:tau+480])**2) for tau in range(1,100)])
            cmnd = np.ones(100)
            cmnd[1:] = difference[1:] * np.arange(1,100) / (np.cumsum(difference[1:])+1e-20)
            troughs = [tau for tau in range(12,98) if cmnd[tau]<cmnd[tau-1] and cmnd[tau]<cmnd[tau+1]]
            good = [tau for tau in troughs if cmnd[tau] < 0.25]
            if good:
                estimates.append(8000/good[0])
        f0 = float(np.median(estimates)) if estimates else None
        # This is a mixed commercial recording; octave errors and instrumental tracking
        # remain possible. Unvoiced words use an explicitly flagged arranged note.
        nominal = f0 if f0 is not None else 330.0
        while nominal < 260.0:
            nominal *= 2.0
        while nominal > 620.0:
            nominal *= 0.5
        midi = round(69+12*math.log2(nominal/440.0))
        pitch = 440.0*2**((midi-69)/12)
        a = audio[round(w.start*16000):round(w.end*16000)]
        energy = float(np.sqrt(np.mean(a*a))) if len(a) else 0.0
        notes.append({"t":round(w.start-start,4), "duration":round(min(0.38,max(0.08,w.end-w.start)),4),
                      "pitch":pitch, "amplitude":min(1.2,max(0.6,energy/0.12)),
                      "pitchMeasured":f0 is not None,"pitchEstimateHz":f0,
                      "alignmentConfidence":w.probability})
    if len(notes) < 8:
        raise ValueError(f"Insufficient aligned events: {len(notes)}")
    result = {"title":"Who Let the Dogs Out — physical bark hook", "sourceUrl":"https://www.youtube.com/watch?v=ojULkWEUsPs",
              "sourceIntervalSeconds":[start,stop],"referenceSha256":hashlib.sha256(args.reference.read_bytes()).hexdigest(),
              "method":"ASR word onsets with YIN estimates on a band-limited mono mix; octave-folded and quantized into 260–620 Hz. Estimated arrangement, not a verified melody transcription. No original audio or lyric text is embedded.",
              "alignmentModel":{"repo":"Systran/faster-whisper-tiny.en","revision":info.sha,"library":"faster-whisper 1.2.1","use":"offline reference timing only; not the voice generator"},
              "notes":notes}
    args.output.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps({"sourceIntervalSeconds":result["sourceIntervalSeconds"],"events":len(notes),"estimatedPitchEvents":sum(n["pitchMeasured"] for n in notes),"alignmentModelRevision":info.sha}))


if __name__ == "__main__":
    main()
