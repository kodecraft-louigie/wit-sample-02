wit_bindgen::generate!();

use exports::local::greeter_demo::greet::Guest;
use crate::exports::local::greeter_demo::greet::SampleStruct;

struct Component02;

impl Guest for Component02{
    fn greetings(_message: SampleStruct) -> SampleStruct {
        let ret = SampleStruct {
            instrument_name: "instrument from comp 02".to_string(),
            ask_iv: 1.0,
            best_ask_amount: 1.0,
            best_ask_price: 1.0,
            bid_iv: 1.0
        };
        ret
    }
}

export!(Component02);
