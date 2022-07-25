# rot8

## automatic display rotation using built-in accelerometer

Automatic rotate modern Linux desktop screen and input devices. Handy for
convertible touchscreen notebooks like HP Spectre x360, Lenovo IdeaPad Flex or Linux phone like Pinephone.

Compatible with [sway](http://swaywm.org/) and [X11](https://www.x.org/wiki/Releases/7.7/).

Available in:

Arch User Repository: [rot8-git](https://aur.archlinux.org/packages/rot8-git/)

Void Package: [rot8](https://github.com/void-linux/void-packages/tree/master/srcpkgs/rot8)

Rust language and the cargo package manager are required to build the binary.

```
$ git clone https://github.com/efernau/rot8
$ cd rot8 && cargo build --release
$ cp target/release/rot8  /usr/bin/rot8
```

or

```
$ cargo install rot8

```

For Sway map your input to the output device:

```

$ swaymsg input <INPUTDEVICE> map_to_output <OUTPUTDEVICE>

```

Call rot8 from sway configuration file ~/.config/sway/config:

```

exec rot8

```

For X11 set Touchscreen Device

```

rot8 --touchscreen <TOUCHSCREEN>

```

there are the following args.

```

--sleep                 // Set sleep millis (500)
--display               // Set Display Device (eDP-1)
--touchscreen           // Set Touchscreen Device X11, allows multiple devices (ELAN0732:00 04F3:22E1)
--keyboard              // Set keyboard to deactivate upon rotation
--threshold             // Set a rotation threshold between 0 and 1 (0.5)
--normalization-factor  // Set factor for sensor value normalization (1e6)
--invert-x              // Invert readings from the HW x axis
--invert-y              // Invert readings from the HW y axis
--invert-z              // Invert readings from the HW z axis
--oneshot               // Updates the screen rotation just once instead of continuously
--version               // Returns the rot8 version

```
