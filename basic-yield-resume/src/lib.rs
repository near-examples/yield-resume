use near_sdk::store::IterableMap;
use near_sdk::{env, near, serde_json, BorshStorageKey, CryptoHash, Gas, GasWeight, PromiseError};

const YIELD_REGISTER: u64 = 0;

#[near]
#[derive(BorshStorageKey)]
enum Keys {
    Map,
}

#[near(serializers = [json])]
pub struct Request {
    id: CryptoHash,
    prompt: String,
}

#[near(contract_state)]
pub struct Contract {
    requests: IterableMap<CryptoHash, String>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            requests: IterableMap::new(Keys::Map),
        }
    }
}

#[near]
impl Contract {
    pub fn request(&mut self, prompt: String) {
        // this will create a unique ID in the YIELD_REGISTER
        let yield_promise = env::promise_yield_create(
            "return_external_response",
            &Vec::new(),
            Gas::from_tgas(5),
            GasWeight::default(),
            YIELD_REGISTER,
        );

        // load the ID created by the promise_yield_create
        let id: CryptoHash = env::read_register(YIELD_REGISTER)
            .expect("read_register failed")
            .try_into()
            .expect("conversion to CryptoHash failed");

        // store the request and wait for the response
        self.requests.insert(id, prompt);
        env::promise_return(yield_promise);
    }

    pub fn respond(&mut self, request_id: CryptoHash, response: String) {
        // resume computation with the response
        env::promise_yield_resume(&request_id, &serde_json::to_vec(&response).unwrap());
    }

    #[private]
    pub fn return_external_response(
        &self,
        #[callback_result] response: Result<String, PromiseError>,
    ) -> String {
        match response {
            Ok(answer) => answer,
            Err(_) => panic!("Timeout while waiting for external response"),
        }
    }

    pub fn list_requests(&self) -> Vec<Request> {
        self.requests
            .iter()
            .map(|(id, prompt)| Request {
                id: id.clone(),
                prompt: prompt.clone(),
            })
            .collect()
    }

    pub fn remove_request(&mut self, id: CryptoHash) {
        self.requests.remove(&id);
    }
}
