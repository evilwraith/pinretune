# PinRetune - PinSound configuration and sound-pack tools

PinRetune is a set of utilities for working with PinSound[^Disclaimer] configuration files
and sound packs. It provides tooling to read and write encrypted PinSound
configuration files, manipulate sound pack IDs, and inspect or transform the
configuration primitives used by PinSound devices.

Most PinSound files use ID scrambling to limit direct modification by end
users. PinRetune provides utilities to read (and write) those files when the
appropriate keys are available, and to translate between scrambled and
unscrambled sound identifiers so authors can work with human-readable IDs.

See the [psconfig](psconfig) crate for details on the supported configuration
primitives and the [psaperio](psaperio) crate for the cryptographic and scrambling
implementation.

## Why this project exists

This project began from a practical need to build and customize a sound pack
for personal hardware. The goal is to provide reliable, well-documented tools
for people who want to author, inspect or adapt PinSound content while
preserving the integrity of original data. PinRetune is intended for users who
already have any necessary keys and legal rights to modify the files they
work with.

#### Note regarding encryption keys

This repository does not contain any encryption keys. Requests for keys or
assistance with obtaining them will not be acted upon.

[^Disclaimer]: This software is not affiliated with, endorsed by, or in any way connected with PinSound, SARL or its affiliates. PinSound is a registered trademark of PinSound, SARL.
