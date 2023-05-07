#![no_main]

use rome_js_parser::parse_script;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let _ = parse_script(&input);
});