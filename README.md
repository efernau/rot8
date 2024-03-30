# rot8

[![Rust](https://github.com/efernau/rot8/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/efernau/rot8/actions/workflows/rust.yml)

## automatic display rotation using built-in accelerometer

Automatically rotate modern Linux desktop screen and input devices. Handy for
convertible touchscreen notebooks like HP Spectre x360, Lenovo IdeaPad Flex
or Linux phone like Pinephone.

Compatible with [X11](https://www.x.org/wiki/Releases/7.7/) and Wayland
compositors which support the `wlr_output_management_v1` protocol (Like
[sway](http://swaywm.org/) and [hyprland](https://hyprland.org/)).

### installation

#### packages

Arch User Repository: [rot8-git](https://aur.archlinux.org/packages/rot8-git/)

GNU Guix Package: [rot8](https://packages.guix.gnu.org/packages/rot8/)

Nixpkgs: [rot8](https://search.nixos.org/packages?show=rot8&type=packages&query=rot8)

Void Package: [rot8](https://github.com/void-linux/void-packages/tree/master/srcpkgs/rot8)



#### manually build from source

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

### usage

Map your inputs to the output device as necessary. e.g. for sway:

```

$ swaymsg input <INPUTDEVICE> map_to_output <OUTPUTDEVICE>

```

Call rot8 from your compositor configuration. e.g. for sway:

```

exec rot8

```

For X11 set Touchscreen Device

```

rot8 --touchscreen <TOUCHSCREEN>

```

This will start the daemon running, continuously checking for rotations.

There are the following args (defaults):

```

--sleep                 // Set millis to sleep between rotation checks (500)
--display               // Set Display Device (eDP-1)
--touchscreen           // Set Touchscreen Device X11, allows multiple devices (ELAN0732:00 04F3:22E1)
--keyboard              // Set keyboard to deactivate upon rotation, for Sway only
--threshold             // Set a rotation threshold between 0 and 1, higher is more sensitive (0.5)
--normalization-factor  // Set factor for sensor value normalization (1e6)
--invert-x              // Invert readings from the HW x axis
--invert-y              // Invert readings from the HW y axis
--invert-z              // Invert readings from the HW z axis
--oneshot               // Updates the screen rotation just once instead of continuously
--beforehooks           // Execute a custom script before rotation
--hooks                 // Execute a custom script after the rotation has finished
--version               // Returns the rot8 version

```

You may need to play with the normalization factor (try multiples of 10) and the axis inversions to get the accelerometer readings to calculate right.
