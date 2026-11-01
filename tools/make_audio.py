#!/usr/bin/env python3
"""Generate original chiptune music and SFX for TAPPER: LAST CALL.

Output: 22.05 kHz mono 16-bit WAV under assets/audio/.
No samples from the arcade game are used.
"""

import math
import random
import struct
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "audio"
SR = 22050


def env(i, n, attack=0.005, release=0.22):
    a = max(1, int(n * attack))
    r = max(1, int(n * release))
    if i < a:
        return i / a
    if i > n - r:
        return max(0.0, (n - i) / r)
    return 1.0


def square(freq, dur, vol=0.22, duty=0.5, slide=None, attack=0.005, release=0.22):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        f = freq if slide is None else freq + (slide - freq) * (i / max(1, n - 1))
        phase += f / SR
        s = vol if (phase % 1.0) < duty else -vol
        out.append(s * env(i, n, attack, release))
    return out


def triangle(freq, dur, vol=0.22, attack=0.005, release=0.22):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        phase += freq / SR
        p = phase % 1.0
        out.append((4 * abs(p - 0.5) - 1) * vol * env(i, n, attack, release))
    return out


def noise(dur, vol=0.2, decay=3.0, lowpass=0.5, seed=1):
    n = int(SR * dur)
    out = []
    rnd = random.Random(seed)
    prev = 0.0
    for i in range(n):
        raw = rnd.uniform(-1, 1)
        prev = prev * lowpass + raw * (1 - lowpass)
        out.append(prev * vol * math.exp(-decay * i / n))
    return out


def mix(*tracks):
    n = max(len(t) for t in tracks)
    out = [0.0] * n
    for t in tracks:
        for i, v in enumerate(t):
            out[i] += v
    return out


def seq(*parts):
    out = []
    for p in parts:
        out.extend(p)
    return out


def rest(dur):
    return [0.0] * int(SR * dur)


def write(name, samples, gain=0.8):
    OUT.mkdir(parents=True, exist_ok=True)
    peak = max(1e-6, max(abs(s) for s in samples))
    scale = min(gain, 0.95) / peak
    data = b"".join(
        struct.pack("<h", int(max(-1, min(1, s * scale)) * 32767)) for s in samples
    )
    path = OUT / name
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(data)
    print(f"{name}: {len(samples) / SR:.2f}s {len(data) // 1024} KiB")


# --------------------------------------------------------------------------
# SFX
# --------------------------------------------------------------------------


def sfx():
    write("pour.wav", mix(
        square(180, 0.30, 0.18, 0.45, slide=90, release=0.5),
        noise(0.30, 0.10, 3.0, 0.75, 3),
        seq(square(520, 0.05, 0.05, 0.2), square(430, 0.05, 0.05, 0.2), rest(0.05)),
    ))
    write("serve.wav", mix(
        square(880, 0.16, 0.16, 0.3, slide=180, release=0.6),
        noise(0.15, 0.12, 6.0, 0.4, 5),
    ))
    write("catch.wav", mix(
        triangle(1568, 0.06, 0.22, release=0.4),
        triangle(2093, 0.09, 0.18, release=0.6),
    ))
    write("empty_catch.wav", seq(
        triangle(1318, 0.05, 0.22, release=0.3),
        triangle(1760, 0.05, 0.20, release=0.3),
        triangle(2637, 0.11, 0.18, release=0.6),
    ))
    write("break.wav", mix(
        noise(0.38, 0.55, 5.0, 0.25, 8),
        seq(square(2400, 0.03, 0.12), square(1800, 0.03, 0.10), square(1200, 0.06, 0.09)),
        square(90, 0.22, 0.20, 0.5, slide=45, release=0.7),
    ))
    write("lane.wav", mix(
        noise(0.10, 0.22, 7.0, 0.7, 11),
        square(110, 0.08, 0.12, 0.5, slide=70, release=0.6),
    ))
    write("served.wav", seq(
        square(523, 0.06, 0.16, 0.5),
        square(659, 0.06, 0.16, 0.5),
        square(784, 0.06, 0.16, 0.5),
        square(1046, 0.14, 0.18, 0.5, release=0.5),
    ))
    write("tip.wav", seq(
        triangle(1046, 0.07, 0.20, release=0.3),
        triangle(1318, 0.07, 0.20, release=0.3),
        triangle(1568, 0.07, 0.20, release=0.3),
        triangle(2093, 0.20, 0.22, release=0.7),
    ))
    write("dancers.wav", seq(
        square(659, 0.08, 0.14, 0.25),
        square(784, 0.08, 0.14, 0.25),
        square(988, 0.08, 0.14, 0.25),
        square(1318, 0.08, 0.14, 0.25),
        rest(0.10),
        square(880, 0.08, 0.14, 0.25),
        square(1108, 0.08, 0.14, 0.25),
        square(1318, 0.16, 0.16, 0.25, release=0.5),
    ))
    write("grab.wav", mix(
        square(420, 0.65, 0.24, 0.5, slide=55, release=0.5),
        noise(0.45, 0.22, 3.0, 0.65, 13),
        square(70, 0.30, 0.18, 0.5, slide=40, release=0.8),
    ))
    write("clear.wav", seq(
        square(523, 0.09, 0.18, 0.5),
        square(659, 0.09, 0.18, 0.5),
        square(784, 0.09, 0.18, 0.5),
        square(1046, 0.18, 0.20, 0.5),
        rest(0.04),
        square(784, 0.09, 0.18, 0.5),
        square(1046, 0.30, 0.22, 0.5, release=0.5),
    ))
    write("game_over.wav", seq(
        square(392, 0.18, 0.20, 0.5),
        square(349, 0.18, 0.20, 0.5),
        square(311, 0.18, 0.20, 0.5),
        square(262, 0.50, 0.22, 0.5, release=0.6),
        triangle(131, 0.60, 0.16, release=0.6),
    ))
    write("bonus_win.wav", seq(
        triangle(784, 0.07, 0.20),
        triangle(988, 0.07, 0.20),
        triangle(1175, 0.07, 0.20),
        triangle(1568, 0.07, 0.20),
        triangle(2093, 0.28, 0.22, release=0.6),
    ))
    write("bonus_lose.wav", mix(
        noise(0.55, 0.42, 2.4, 0.8, 21),
        seq(square(392, 0.10, 0.16), square(311, 0.10, 0.16), square(233, 0.28, 0.18)),
    ))
    write("start.wav", seq(
        triangle(1046, 0.05, 0.18),
        triangle(1318, 0.05, 0.18),
        triangle(1568, 0.12, 0.20, release=0.5),
        noise(0.06, 0.08, 6.0, 0.5, 2),
    ))
    write("extra.wav", seq(
        square(784, 0.08, 0.18, 0.4),
        square(988, 0.08, 0.18, 0.4),
        square(1175, 0.08, 0.18, 0.4),
        square(1568, 0.24, 0.20, 0.4, release=0.5),
    ))


# --------------------------------------------------------------------------
# Music
# --------------------------------------------------------------------------

MIDI = {
    "C3": 48, "D3": 50, "E3": 52, "F3": 53, "G3": 55, "A3": 57, "B3": 59,
    "C4": 60, "D4": 62, "E4": 64, "F4": 65, "G4": 67, "A4": 69, "B4": 71,
    "C5": 72, "D5": 74, "E5": 76, "F5": 77, "F#5": 78, "G5": 79, "A5": 81, "B5": 83,
    "C6": 84,
    "Bb3": 58, "Eb4": 63, "F#4": 66, "Bb4": 70, "Ab4": 68, "Db4": 61,
}


def freq(midi):
    return 440.0 * (2 ** ((midi - 69) / 12))


def tone(midi, beats, bpm, wave="square", vol=0.16, duty=0.5):
    dur = beats * 60.0 / bpm
    f = freq(MIDI[midi])
    if wave == "triangle":
        return triangle(f, dur, vol)
    if wave == "square25":
        return square(f, dur, vol, 0.25)
    return square(f, dur, vol, duty)


def play(notes, bpm, wave="square", vol=0.16, duty=0.5):
    out = []
    for midi, beats in notes:
        if midi is None:
            out.extend(rest(beats * 60.0 / bpm))
        else:
            out.extend(tone(midi, beats, bpm, wave, vol, duty))
    return out


def drums(bpm, bars, pattern="kick_snare", vol=0.10):
    beat = 60.0 / bpm
    out = []
    for bar in range(bars):
        for b in range(4):
            if pattern == "swing":
                out.extend(noise(0.035, vol * 0.45, 9.0, 0.8, 30 + bar * 4 + b))
                out.extend(rest(beat * 0.20))
                if b % 2 == 0:
                    out.extend(square(80, 0.07, vol * 0.7, 0.5, slide=45, release=0.5))
                    out.extend(rest(beat * 0.20))
                else:
                    out.extend(rest(beat * 0.30))
            else:
                if b in (0, 2):
                    out.extend(square(90, 0.06, vol * 0.8, 0.5, slide=45, release=0.5))
                    out.extend(rest(beat - 0.06))
                else:
                    out.extend(noise(0.05, vol * 0.9, 7.0, 0.7, 40 + bar * 4 + b))
                    out.extend(rest(beat - 0.05))
    return out


def fit(track, beats, bpm):
    n = int(beats * 60.0 / bpm * SR)
    if len(track) < n:
        track = track + [0.0] * (n - len(track))
    return track[:n]


def music_saloon():
    bpm = 138
    mel = [
        ("C5", 1), ("E5", 1), ("G5", 1.5), ("E5", .5),
        ("F5", 1.5), ("E5", .5), ("D5", 1), ("C5", 1),
        ("G4", 1), ("C5", 1), ("E5", 1), ("D5", 1),
        ("C5", 1), ("A4", 1), ("G4", 1), ("A4", 1),
        ("E5", 1), ("G5", 1), ("A5", 1.5), ("G5", .5),
        ("E5", 1), ("D5", .5), ("C5", .5), ("A4", 2),
        ("F5", 1), ("E5", 1), ("D5", 1), ("C5", 1),
        ("G4", 1), ("A4", 1), ("C5", 2),
    ]
    bass = [
        ("C3", 1), ("G3", 1), ("C3", 1), ("G3", 1),
        ("F3", 1), ("C4", 1), ("F3", 1), ("C4", 1),
        ("C3", 1), ("G3", 1), ("C3", 1), ("G3", 1),
        ("G3", 1), ("D3", 1), ("G3", 1), ("D3", 1),
        ("A3", 1), ("E3", 1), ("A3", 1), ("E3", 1),
        ("F3", 1), ("C4", 1), ("F3", 1), ("C4", 1),
        ("C3", 1), ("G3", 1), ("C3", 1), ("G3", 1),
        ("G3", 1), ("D3", 1), ("G3", 1), ("D3", 1),
    ]
    melody = fit(play(mel, bpm, "square", 0.15, 0.35), 32, bpm)
    low = fit(play(bass, bpm, "triangle", 0.16), 32, bpm)
    beat = fit(drums(bpm, 8, "swing", 0.075), 32, bpm)
    return mix(melody, low, beat)


def music_sports():
    bpm = 152
    mel = [
        ("G4", .5), ("G4", .5), ("B4", .5), ("D5", .5), ("G5", 1), ("D5", 1),
        ("E5", 1), ("D5", .5), ("B4", .5), ("C5", 1), ("B4", 1),
        ("G4", .5), ("G4", .5), ("B4", .5), ("D5", .5), ("G5", 1), ("A5", 1),
        ("G5", .5), ("F#5", .5), ("E5", 1), ("D5", 1), ("G4", 1),
        ("C5", .5), ("E5", .5), ("G5", 1), ("E5", .5), ("C5", .5), ("A4", 1),
        ("D5", .5), ("F#5", .5), ("A5", 1), ("F#5", .5), ("D5", .5), ("B4", 1),
        ("G5", 1), ("E5", 1), ("C5", 1), ("A4", 1),
        ("B4", 1), ("D5", 1), ("G4", 2),
    ]
    bass = [
        ("G3", 1), ("G3", 1), ("D3", 1), ("D3", 1),
        ("C3", 1), ("C3", 1), ("G3", 1), ("G3", 1),
        ("G3", 1), ("G3", 1), ("D3", 1), ("D3", 1),
        ("C3", 1), ("D3", 1), ("G3", 1), ("G3", 1),
        ("A3", 1), ("A3", 1), ("E3", 1), ("E3", 1),
        ("D3", 1), ("D3", 1), ("A3", 1), ("A3", 1),
        ("C3", 1), ("C3", 1), ("G3", 1), ("G3", 1),
        ("D3", 1), ("D3", 1), ("G3", 1), ("G3", 1),
    ]
    melody = fit(play(mel, bpm, "square", 0.13, 0.25), 32, bpm)
    low = fit(play(bass, bpm, "square", 0.12, 0.5), 32, bpm)
    beat = fit(drums(bpm, 8, "march", 0.09), 32, bpm)
    return mix(melody, low, beat)


def music_punk():
    bpm = 176
    mel = [
        ("E5", .5), ("E5", .5), ("G5", .5), ("E5", .5), ("A5", .5), ("G5", .5), ("E5", .5), ("D5", .5),
        ("E5", .5), ("E5", .5), ("B5", .5), ("A5", .5), ("G5", 1), ("E5", 1),
        ("C5", .5), ("C5", .5), ("E5", .5), ("C5", .5), ("G5", .5), ("F5", .5), ("E5", .5), ("C5", .5),
        ("D5", .5), ("E5", .5), ("G5", 1), ("B4", 1), ("E5", 1),
        ("E5", .5), ("E5", .5), ("G5", .5), ("E5", .5), ("A5", .5), ("G5", .5), ("E5", .5), ("D5", .5),
        ("B5", .5), ("A5", .5), ("G5", .5), ("E5", .5), ("D5", 1), ("E5", 1),
        ("A4", .5), ("C5", .5), ("E5", .5), ("A5", .5), ("G5", .5), ("E5", .5), ("C5", .5), ("A4", .5),
        ("B4", .5), ("D5", .5), ("F#5", 1), ("E5", 2),
    ]
    bass = [
        ("E3", 1), ("E3", 1), ("D3", 1), ("D3", 1),
        ("C3", 1), ("C3", 1), ("B3", 1), ("B3", 1),
        ("E3", 1), ("E3", 1), ("D3", 1), ("D3", 1),
        ("C3", 1), ("B3", 1), ("E3", 1), ("E3", 1),
        ("E3", 1), ("E3", 1), ("D3", 1), ("D3", 1),
        ("C3", 1), ("C3", 1), ("B3", 1), ("B3", 1),
        ("A3", 1), ("A3", 1), ("B3", 1), ("B3", 1),
        ("E3", 1), ("E3", 1), ("E3", 1), ("E3", 1),
    ]
    melody = fit(play(mel, bpm, "square", 0.12, 0.15), 32, bpm)
    low = fit(play(bass, bpm, "square", 0.13, 0.5), 32, bpm)
    beat = fit(drums(bpm, 8, "punk", 0.10), 32, bpm)
    return mix(melody, low, beat)


def music_alien():
    bpm = 104
    mel = [
        ("A4", 2), ("C5", 1), ("E5", 1),
        ("D5", 1.5), ("C5", .5), ("B4", 2),
        ("F4", 2), ("A4", 1), ("C5", 1),
        ("E5", 1.5), ("D5", .5), ("C5", 2),
        ("G4", 1), ("B4", 1), ("D5", 1), ("F5", 1),
        ("E5", 1.5), ("D5", .5), ("C5", 1), ("B4", 1),
        ("A4", 1), ("C5", 1), ("E5", 1), ("A5", 1),
        ("G5", 1.5), ("E5", .5), ("D5", 1), ("A4", 1),
    ]
    bass = [
        ("A3", 2), ("E3", 2),
        ("F3", 2), ("C3", 2),
        ("G3", 2), ("D3", 2),
        ("E3", 2), ("A3", 2),
        ("A3", 2), ("E3", 2),
        ("F3", 2), ("C3", 2),
        ("G3", 2), ("D3", 2),
        ("E3", 2), ("A3", 2),
    ]
    melody = fit(play(mel, bpm, "triangle", 0.16), 32, bpm)
    low = fit(play(bass, bpm, "square", 0.10, 0.5), 32, bpm)
    shimmer = fit(seq(*[
        tone("E5" if i % 2 else "A5", 1, bpm, "triangle", 0.035)
        for i in range(16)
    ]), 32, bpm)
    beat = fit(drums(bpm, 8, "swing", 0.05), 32, bpm)
    return mix(melody, low, shimmer, beat)


def main():
    sfx()
    write("music_saloon.wav", music_saloon(), gain=0.72)
    write("music_sports.wav", music_sports(), gain=0.72)
    write("music_punk.wav", music_punk(), gain=0.72)
    write("music_alien.wav", music_alien(), gain=0.72)


if __name__ == "__main__":
    main()
