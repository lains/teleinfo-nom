# teleinfo\_nom: a wannabe comprehensive teleinfo parser in nom

## Purpose

This crate aims at parsing data from a teleinfo bus from french power meters.
Information for the data format is available in french for [linky](https://www.enedis.fr/sites/default/files/Enedis-NOI-CPT_54E.pdf) and [older meters](https://www.enedis.fr/sites/default/files/Enedis-NOI-CPT_02E.pdf).
This crate parses only personal customers or small business contracts (blue contract).

If you need a smaller crate you can use [teleinfo-parser](https://crates.io/crates/teleinfo-parser).

## Status

The crate allows to access all field from a legacy or standard message. It includes helper functions to get values from the message like current tarif indices or return the matcing indices for legacy contract. The mode of the message is autodetected.

## Todo

Getting the same info for standard messages than legacy for billing indices will need more information but could be done.
Parsing of binary fields could be easily done like STGE fields in standard mode.

## Usage

```
use std::fs::File;
extern crate teleinfo_nom;
// Could be a serial port with serialport crate
let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
let (remain, msg1) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
let current_indices = msg1.get_billing_indices();
let current_values = msg1.get_values(current_indices);
for (index,value) in current_values.into_iter() {
  match value {
    Some(val) => println!("store {}: {} in database", index, val),
    None => (),
  }
}
let (remain, msg2) = teleinfo_nom::get_message(&mut stream, remain).unwrap();
```
