# car-demo

## controls

### config

config flags can be viewed with the `--help` flag

### keyboard

- mouse controls hand
- left button to grab
- w to accelerate
- s to brake
- shift to push down clutch
- esc to quit

### controller

- right stick controls hand
- right shoulder to grab stick
- left shoulder to push down clutch
- right trigger to accelerate
- left trigger to brake

### quirks

there is a box for the gears to be recognized as "in", which it is possible to miss; this is to emulate the feeling of sometimes missing a gear on the cusp if it wasn't entirely in.

as a result of this, it'll only try to reset you to the center provided you are not in a state of missing a gear between neutral and some other gear.

you aren't able to move from slot to slot without moving down to the center first, due to gears being as such:
```
. . .
| | |
|-|-|
| | |
' ' '
```

however sometimes it's possible to skip between "walls". this is a bug.

## dependencies

`sdl2`
`sdl2_gfx`
`sdl2_image`

### arch installation:

`pacman -S sdl2 sdl2_gfx sdl2_image`

## todo

- functioning rpm/speedometer
