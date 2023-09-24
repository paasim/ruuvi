# ruuvi

A rust package for obtaining data from ruuvitags via bluetooth.

Usage
-----

    # print first rawv5 advertisement from each ruuvitag as json
    # until one observation is printed from all of them
    cargo run -r -- --macs AB:CD:EF:12:34:56 78:90:AB:CD:EF:12

    # print rawv5 advertisements indefinitely
    cargo run -r
