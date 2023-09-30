# ruuvi

[![build](https://github.com/paasim/ruuvi/workflows/build/badge.svg)](https://github.com/paasim/ruuvi/actions)

A rust package for [communicating with ruuvitags via bluetooth](https://docs.ruuvi.com/).

Dependencies
------------

Linux-only as it uses [BlueZ](https://github.com/bluez/bluez) for bluetooth and [BlueR](https://crates.io/crates/bluer) specifically as a rust-dependecy. `BlueR` also uses depends on `libdbus`.

Some of the functionality (of `BlueR`) uses experimental B-Bus interfaces and might require `Experimental = true` in `/etc/bluetooth.main.conf`.

Usage
-----

    # print first rawv5 advertisement from each device as json
    # until one observation is printed from all of them
    cargo run -r -- --latest AB:CD:EF:12:34:56 78:90:AB:CD:EF:12

    # print rawv5 advertisements indefinitely
    cargo run -r

    # print observation log for the last 2 (ruuvitags support at most 10) days
    cargo run -r -- --log AB:CD:EF:12:34:56 2
