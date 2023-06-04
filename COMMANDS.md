# Pinsound configuration syntax
Pinsound configuration files support a list of commands delimited by a pipe ('|) character. The configuration file seems to have a trailing null byte like a typical c-string however works just fine without one.

There are multiple types of configuration files and supported primitives may differ depending on configuration file.

### Checksum
The checksum "seed" used to scramble sound ids. Each pin has it's own unique checksum. Some pins have multiple, presumably to combat conversion tables.
```
checksum <seed>
example: checksum TheSoprOrig
```

### Checksums
Defines the list of valid checksums for a specific pin. This primitive is normally only seen in firmware configuration files but can be used everywhere. This means it can be used for giggles to generate a soundpack with a custom checksum.
```
checksums <name> ...
example: checksums SomeChecksum AnotherChecksum
```

### Shaker presets
Receiving sound command `id`, will initiate shaker preset `preset`.
```
<id> shakerPreset <preset>
example: 1080 shakerPreset 2
```

### Stop/cancel playback of sounds
Receiving sound command `id1`, playback of sound `id2` will be cancelled.
```
<id1> stop <id2>
example: 120 stop 33
```

### RTTTL - Ring Tone Text Transfer Language
Defines the tune that is played by the shaker at boot. See Wikipedia for more on [RTTTL](https://en.wikipedia.org/wiki/Ring_Tone_Text_Transfer_Language)
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


