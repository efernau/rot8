# rot8

## automatic display rotation

Automatic rotate modern Linux desktop screen and input devices. Handy for
convertible touchscreen notebooks like the Kaby Lake model of the HP Spectre x360.

Rust language and the cargo package manager are required to build the binary.

```
$ git clone https://github.com/efernau/rot8
$ cd rot8 && cargo build --release
$ cp target/release/rot8  /usr/bin/rot8
```

Call Rote8 from sway configuration file ~/.config/sway/config:

```
exec rot8
```
