use near_sdk::store::IterableMap;
use near_sdk::{near, serde_json, BorshStorageKey, Gas, GasWeight, Promise, PromiseError, YieldId};
use serde_json::json;

#[near]
#[derive(BorshStorageKey)]
enum Keys {
    Map,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Request {
    yield_id: YieldId,
    prompt: String,
}

#[near(serializers = [json])]
pub enum Response {
    Answer(String),
    TimeOutError,
}

#[near(contract_state)]
pub struct Contract {
    request_id: u32,
    requests: IterableMap<u32, Request>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            request_id: 0,
            requests: IterableMap::new(Keys::Map),
        }
    }
}

#[near]
impl Contract {
    pub fn request(&mut self, prompt: String) -> Promise {
        // internal variable to keep track of the requests
        self.request_id += 1;

        // create the yielded promise and its resumption ID
        let (yield_promise, yield_id) = Promise::new_yield(
            "return_external_response",
            &json!({ "request_id": self.request_id })
                .to_string()
                .into_bytes(),
            Gas::from_tgas(5),
            GasWeight::default(),
        );

        // store the request, so we can delete it later
        let request = Request { yield_id, prompt };
        self.requests.insert(self.request_id, request);

        // return the yield promise
        yield_promise
    }

    pub fn respond(&mut self, yield_id: YieldId, response: String) {
        // resume computation with the response
        yield_id
            .resume(serde_json::to_vec(&response).unwrap())
            .expect("yielded promise not found or already resolved");
    }

    #[private]
    pub fn return_external_response(
        &mut self,
        request_id: u32,
        #[callback_result] response: Result<String, PromiseError>,
    ) -> Response {
        self.requests.remove(&request_id);

        match response {
            Ok(answer) => Response::Answer(answer),
            Err(_) => Response::TimeOutError,
        }
    }

    pub fn list_requests(&self) -> Vec<Request> {
        self.requests
            .values()
            .map(|request| request.clone())
            .collect()
    }
}
