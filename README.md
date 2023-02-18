# Shark Emulator (Chip8 - Rust version)

*Base on [SharkE-Chip8-CSharp (Sdl2 version)](https://github.com/frcs6/SharkE-Chip8-CSharp)*

This is a Chip8 emulator implemented in Rust using [SDL](https://www.libsdl.org/) render. [SDL](https://www.libsdl.org/) mapping use [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2).

*Only Windows executable has sound driver implemented*

## Command line arguments

```
sharke-chip8 [rom]
```

## Special keys

```
[ESC] : Exit game
```
## Keyboard
Chip8 layout:
| 1 | 2 | 3 | C |
|---|---|---|---|
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

Emulator mapping:
| 1 | 2 | 3 | 4 |
|---|---|---|---|
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

## Documentations
 - https://en.wikipedia.org/wiki/CHIP-8
 - http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
 - https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/

## Tests
 - https://github.com/corax89/chip8-test-rom
 - https://github.com/Skosulor/c8int/tree/master/test
 
 ## Roms
  - https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html
