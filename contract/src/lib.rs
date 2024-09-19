use near_sdk::store::IterableMap;
use near_sdk::{env, near, serde_json, BorshStorageKey, CryptoHash, Gas, GasWeight, PromiseError};
use serde_json::json;

const YIELD_REGISTER: u64 = 0;

#[near]
#[derive(BorshStorageKey)]
enum Keys {
    Map,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Request {
    yield_id: CryptoHash,
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
    pub fn request(&mut self, prompt: String) {
        // internal variable to keep track of the requests
        self.request_id += 1;

        // this will create a unique ID in the YIELD_REGISTER
        let yield_promise = env::promise_yield_create(
            "return_external_response",
            &json!({ "request_id": self.request_id })
                .to_string()
                .into_bytes(),
            Gas::from_tgas(5),
            GasWeight::default(),
            YIELD_REGISTER,
        );

        // load the ID created by the promise_yield_create
        let yield_id: CryptoHash = env::read_register(YIELD_REGISTER)
            .expect("read_register failed")
            .try_into()
            .expect("conversion to CryptoHash failed");

        // store the request, so we can delete it later
        let request = Request { yield_id, prompt };
        self.requests.insert(self.request_id, request);

        // return the yield promise
        env::promise_return(yield_promise);
    }

    pub fn respond(&mut self, yield_id: CryptoHash, response: String) {
        // resume computation with the response
        env::promise_yield_resume(&yield_id, &serde_json::to_vec(&response).unwrap());
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
