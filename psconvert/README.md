# PSConvert

PSConvert turns a PinSound[^PinSound] sound pack into an AltSound package for
[libaltsound](https://github.com/vpinball/libaltsound), the AltSound player in
PinMAME and Visual Pinball standalone. The folder IDs of a pack are scrambled with
its `checksum` (see [psaperio](../psaperio)); the package lists the plain sound
command IDs the pinball's CPU board sends.

```
pinretune list    <pack>
pinretune convert <pack> <pinmame>/altsound/<rom> [--format gsound|altsound] [--target-db -19] [--no-normalize]
```

`list` prints every sound with its command ID, group and folder. `convert` writes a
new package folder: the audio files (copied unchanged, flat, named
`0x<command>-<label>.wav`), `g-sound.csv` or `altsound.csv`, and `altsound.ini`. Name
the folder after the ROM PinMAME runs; the pack's `roms` statement lists the ones it
was made for.

## Sound packs

- **Groups:** up to five group folders, `music`, `jingle`, `single`, `sfx` and `voice`. The names are case-sensitive; any other folder is ignored.
- **Sounds:** each sound is a folder `<id>-<label>` holding one or more WAV files. With several, one is picked at random per play. The label is only a label.
- **IDs:** a pack whose largest ID is 65535 or less uses plain command IDs (the older 6-digit layout). Otherwise every ID above 65535 is scrambled with the `checksum`.
- **Levels and ducking:** `gain <group> N` and `ducking <group> N` in `config.pinsound`, or `gain.txt` / `ducking.txt` in a group folder.
  - Gains are linear with 10 = unity. A sound's gain (`gain N` in its own `config.pinsound`, or `gain.txt` in its folder) times its group's gain gives its volume, capped at full scale.
  - Ducking is the percentage of music and single volume kept while a sound of that group plays. Defaults: jingle 5, sfx 90, voice 85.

## How PinSound behaviour maps

| PinSound | G-Sound (`--format gsound`, default) | AltSound (`--format altsound`) |
|---|---|---|
| music: one at a time, loops; a new one also stops a single | MUSIC, `[music] stops = solo` | channel 0, LOOP |
| single: one at a time; stops music | SOLO, `[solo] stops = music` | channel 1, STOP |
| jingle: one at a time; ducks music and single | OVERLAY, ducking profile 1 | channel 1, DUCK |
| voice: ducks music and single | CALLOUT, ducking profile 1 | channel -1, DUCK |
| sfx: ducks music and single | SFX, ducking profile 1 | channel -1, DUCK |

G-Sound is the closer match: each group has its own type, a single is ducked and
stopped like music, and the ducking percentages live in the ini once.

**Not carried over** by either format; `convert` lists what a pack uses of these:
- the `max` polyphony limits (2 voices, 3 sfx);
- the gradual ducking ramp;
- start delays;
- `<id> stop` / `donotstop` / `playif` rules;
- the Data East music stack.

## Loudness

PinSound packs are mastered at very different levels; one measured pack averages
12 dB below another. On PinSound hardware the board's amplifier makes up for it. In
libaltsound a sample's GAIN stops at 100, so `convert` measures the pack's average
loudness (the mean volume of each file, as ffmpeg's `volumedetect` reports it,
averaged) and writes the difference to `--target-db` into `altsound.ini` as
`[system] package_gain_db`.
- The gain only ever raises a pack, never lowers it.
- The key needs a libaltsound with package gain support, which also limits the mix so the extra gain cannot clip. Older versions ignore it.
- `--no-normalize` writes 0.

[^PinSound]: This software is not affiliated, associated, authorized, endorsed by, or in any way connected with PinSound, SARL, or any of its subsidiaries or its affiliates.
  PinSound is a registered trademark of PinSound, SARL.
