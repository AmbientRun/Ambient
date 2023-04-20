#![allow(dead_code)]

pub const SOUNDS: [(&str, &str); 8] = [
    ("Kick Drum", "assets/BD2500.ogg"),
    ("Snare Drum", "assets/SD7550.ogg"),
    ("Closed Hihat", "assets/CH.ogg"),
    ("Open Hihat", "assets/OH75.ogg"),
    ("Low Conga", "assets/LC00.ogg"),
    ("Mid Conga", "assets/MC00.ogg"),
    ("High Tom", "assets/HT75.ogg"),
    ("Mid Tom", "assets/MT75.ogg"),
];

pub const BEAT_COUNT: usize = 16;
pub const NOTE_COUNT: usize = SOUNDS.len() * BEAT_COUNT;
pub const BEATS_PER_MINUTE: usize = 120;
pub const SECONDS_PER_BEAT: f32 = 60.0 / BEATS_PER_MINUTE as f32;
pub const SECONDS_PER_NOTE: f32 = SECONDS_PER_BEAT / 4.0;
