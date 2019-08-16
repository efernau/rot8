# rot8

## automatic display rotation using built-in accelerometer

Automatic rotate modern Linux desktop screen and input devices. Handy for
convertible touchscreen notebooks like the Kaby Lake model of the HP Spectre x360.

Compatible with [sway](http://swaywm.org/) and [X11](https://www.x.org/wiki/Releases/7.7/).

Rust language and the cargo package manager are required to build the binary.

```
$ git clone https://github.com/efernau/rot8
$ cd rot8 && cargo build --release
$ cp target/release/rot8  /usr/bin/rot8
```

For Sway map your input to the output device:

```
$ swaymsg <INPUTDEVICE> map_to_output <OUTPUTDEVICE>
```

Call Rote8 from sway configuration file ~/.config/sway/config:

```
exec rot8
```

For X11 set Touchscreen Device

```
rot8 --touchscreen <TOUCHSCREEN>
```

there are the following args.

```
--sleep         // Set sleep millis (500)
--display       // Set Display Device (eDP-1)
--touchscreen   // Set Touchscreen Device X11 (ELAN0732:00 04F3:22E1)
```
