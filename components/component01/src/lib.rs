wit_bindgen::generate!();

use std::io::Read;

use exports::wasi::http::incoming_handler::Guest as HttpServerTrait;
use wasi::http::types::*;

use exports::wasmcloud::messaging::handler::Guest as MessagingTrait;
use wasmcloud::messaging::*;

use wasi::logging::logging::*;
use serde::{Deserialize, Serialize};

//use local::greeter_demo::greet::SampleStruct;

struct Component01;

impl HttpServerTrait for Component01 {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) { 

        // let _ = consumer::publish(&types::BrokerMessage {
        //         subject: "wasmcloud.component01".to_string(),
        //         reply_to: None,
        //         body: "Publish Message ko!!!".to_string().into_bytes(),
        //     });
        
        log(Level::Info,"","publish success!");      
             
        let _x =  _request.consume().unwrap();
        // let req = SampleStruct{
        //     instrument_name: "request ko".to_string(),
        //     ask_iv: 1.0,
        //     best_ask_amount: 1.0,
        //     best_ask_price: 1.0,
        //     bid_iv: 1.0
        // };
        
        //let pong = local::greeter_demo::greet::greetings(&req); // to comp02     
        // let ret = Response {
        //     instrument: pong.instrument_name,
        //     ask_iv: pong.ask_iv,
        //     best_ask_amount: pong.best_ask_amount,
        //     best_ask_price: pong.best_ask_price,
        //     bid_iv: pong.bid_iv
        // };     
        //let json_string = serde_json::to_string(&ret).unwrap();

        // Build a request to dog.ceo which returns a URL at which we can find a doggo
        let req = wasi::http::outgoing_handler::OutgoingRequest::new(Fields::new());
        req.set_scheme(Some(&Scheme::Https)).unwrap();
        req.set_authority(Some("dog.ceo")).unwrap();
        req.set_path_with_query(Some("/api/breeds/image/random"))
            .unwrap();

        // Perform the API call to dog.ceo, expecting a URL to come back as the response body
        let dog_picture_url = match wasi::http::outgoing_handler::handle(req, None) {
            Ok(resp) => {
                resp.subscribe().block();
                let response = resp
                    .get()
                    .expect("HTTP request response missing")
                    .expect("HTTP request response requested more than once")
                    .expect("HTTP request failed");
                if response.status() == 200 {
                    let response_body = response
                        .consume()
                        .expect("failed to get incoming request body");
                    let body = {
                        let mut buf = vec![];
                        let mut stream = response_body
                            .stream()
                            .expect("failed to get HTTP request response stream");
                        InputStreamReader::from(&mut stream)
                            .read_to_end(&mut buf)
                            .expect("failed to read value from HTTP request response stream");
                        buf
                    };
                    let _trailers = wasi::http::types::IncomingBody::finish(response_body);
                    let dog_response: DogResponse = serde_json::from_slice(&body).unwrap();
                    dog_response.message
                } else {
                    format!("HTTP request failed with status code {}", response.status())
                }
            }
            Err(e) => {
                format!("Got error when trying to fetch dog: {}", e)
            }
        };

       
        let h: Headers = Fields::new();
        let _ = h.set(&"content-Type".to_string(), &["application/json".to_string().into_bytes()]);
        let response = OutgoingResponse::new(h);
        response.set_status_code(200).unwrap();       
        let response_body = response.body().unwrap();
        ResponseOutparam::set(response_out, Ok(response));
        response_body
            .write()
            .unwrap()
            .blocking_write_and_flush(dog_picture_url.as_bytes())
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

#[derive(serde::Deserialize)]
struct DogResponse {
    message: String,
}

pub struct InputStreamReader<'a> {
    stream: &'a mut crate::wasi::io::streams::InputStream,
}

impl<'a> From<&'a mut crate::wasi::io::streams::InputStream> for InputStreamReader<'a> {
    fn from(stream: &'a mut crate::wasi::io::streams::InputStream) -> Self {
        Self { stream }
    }
}

impl std::io::Read for InputStreamReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use crate::wasi::io::streams::StreamError;
        use std::io;

        let n = buf
            .len()
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        match self.stream.blocking_read(n) {
            Ok(chunk) => {
                let n = chunk.len();
                if n > buf.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "more bytes read than requested",
                    ));
                }
                buf[..n].copy_from_slice(&chunk);
                Ok(n)
            }
            Err(StreamError::Closed) => Ok(0),
            Err(StreamError::LastOperationFailed(e)) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_debug_string()))
            }
        }
    }
}

export!(Component01);