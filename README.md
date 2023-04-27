# PinResound - Uncovering the obscured
A tool to assist with encryption, decryption, scrambling and unscrambling of all things PinSound.

## What is PinSound?
If you are active in the pinball community you most likely already know. PinSound is a suite of electronics and hardware designed to enhance the pinball experience. This repository and tooling will focus on the sound cards specifically, which by the way are great products!

### What is PinResound?

## Pinsound commands

### Checksums
The checksum "seed" used to scramble sound ids. Each pin has it's own unique checksum. Some pins have multiple, presumably to combat conversion tables.
```
checksums <seed>
example: checksums TheSoprOrig
```

### Shaker presets
Receiving sound `id` command, will initiate shaker preset `preset`.
```
<id> shakerPreset <preset>
example: 1080 shakerPreset 2
```

### Stop/cancel playback of sounds
Receiving sound `id1` command, playback of sound `id2` will be cancelled.
```
<id1> stop <id2>
example: 120 stop 33
```

### RTTTL - Ring Tone Text Transfer Language
Unknown usage. See Wikipedia for more on [RTTTL](https://en.wikipedia.org/wiki/Ring_Tone_Text_Transfer_Language)
```
rtttl <data>
example: rtttl Addams Family :d=4, o=6, b=50 :32p, 32c#, 16f#, 32a#, 16f#, 32c#, 16c, 8g#, 32f#, 16f, 32g#, 16f, 32c#, 16a#5, 8f#
```


## Unknown commands
`auto_mute 600`

`cpusyncaddr 202 190 11 330 12 330 11 130 12`

`cpusync music 3F BF sumaddrmusic 80 800`

`DE_main_music_disabled 1`

`4 delay 400`

`donotstop`

`donotplayif`

`playif`

`playif_else_stopall`

`ping_03d3 3F BF 3F`

`shakerCustomPresetStart C 50`

`shakerCustomPresetEnd A 100`

`shakerCustomPresetDuration B 4000`

# Disclaimer
This software is not affiliated, associated, authorized, endorsed by, or in any way
connected with PinSound, SARL, or any of its subsidiaries or its affiliates.

PinSound is a registered trademark of PinSound, SARL.