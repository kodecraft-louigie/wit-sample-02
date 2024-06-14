wit_bindgen::generate!();

use exports::wasi::http::incoming_handler::Guest as HttpServerTrait;
use wasi::http::types::*;

use exports::wasmcloud::messaging::handler::Guest as MessagingTrait;
use wasmcloud::messaging::*;

use wasi::logging::logging::*;
use serde::{Deserialize, Serialize};

use local::greeter_demo::greet::SampleStruct;

struct Component01;

impl HttpServerTrait for Component01 {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) { 

        let _ = consumer::publish(&types::BrokerMessage {
                subject: "wasmcloud.component01".to_string(),
                reply_to: None,
                body: "Publish Message ko!!!".to_string().into_bytes(),
            });
        log(Level::Info,"","publish success!");      
             
        let _x =  _request.consume().unwrap();
        let req = SampleStruct{
            instrument_name: "request ko".to_string(),
            ask_iv: 1.0,
            best_ask_amount: 1.0,
            best_ask_price: 1.0,
            bid_iv: 1.0
        };
        
        let pong = local::greeter_demo::greet::greetings(&req); // to comp02
        
        let ret = Response {
            instrument: pong.instrument_name,
            ask_iv: pong.ask_iv,
            best_ask_amount: pong.best_ask_amount,
            best_ask_price: pong.best_ask_price,
            bid_iv: pong.bid_iv
        };
       
        let json_string = serde_json::to_string(&ret).unwrap();
        
        let h: Headers = Fields::new();
        let _ = h.set(&"content-Type".to_string(), &["application/json".to_string().into_bytes()]);
        let response = OutgoingResponse::new(h);
        response.set_status_code(200).unwrap();       
        let response_body = response.body().unwrap();
        ResponseOutparam::set(response_out, Ok(response));
        response_body
            .write()
            .unwrap()
            .blocking_write_and_flush(json_string.as_bytes())
            .unwrap();
        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    }
}


impl MessagingTrait for Component01 {
    fn handle_message(msg: types::BrokerMessage) -> Result<(), String> {
        let string = String::from_utf8(msg.body);
        if !string.clone().unwrap().is_empty() {
            log(Level::Info,"",format!("Received message: {}", string.unwrap()).as_str()); 
            Ok(()) 
        } else {
            log(
                Level::Warn,
                "",
                "No reply_to field in message, ignoring message",
            );
            Ok(())
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub instrument: String,
    pub ask_iv: f64,
    pub best_ask_amount: f64,
    pub best_ask_price: f64,
    pub bid_iv: f64
}

export!(Component01);