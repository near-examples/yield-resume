use near_sdk::store::IterableMap;
use near_sdk::{
    env, near, serde_json, BorshStorageKey, CryptoHash, Gas, GasWeight, NearToken, PromiseError,
};

type Prompt = String;
const YIELD_REGISTER: u64 = 0;

#[near]
#[derive(BorshStorageKey)]
enum Keys {
    Requests,
    Answers,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub enum Response {
    Wait,
    Answer(String),
}

#[near(serializers = [json])]
pub struct Request {
    id: CryptoHash,
    prompt: Prompt,
}

#[near(contract_state)]
pub struct Contract {
    requests: IterableMap<CryptoHash, Prompt>,
    answers: IterableMap<Prompt, Response>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            requests: IterableMap::new(Keys::Requests),
            answers: IterableMap::new(Keys::Answers),
        }
    }
}

#[near]
impl Contract {
    pub fn request(&mut self, prompt: Prompt) {
        // User might call back if there was a timeout
        let existing_return = env::promise_create(
            env::current_account_id(),
            "return_existing_response",
            &serde_json::to_vec(&(&prompt,)).unwrap(),
            NearToken::from_near(0),
            Gas::from_tgas(5),
        );

        match self.answers.get(&prompt) {
            None => (),
            Some(Response::Wait) => panic!("Please try again in some seconds"),
            Some(Response::Answer(_)) => return env::promise_return(existing_return),
        }

        // A new request, lets create a unique ID in the YIELD_REGISTER
        let yield_promise = env::promise_yield_create(
            "return_external_response",
            &serde_json::to_vec(&(&prompt,)).unwrap(),
            Gas::from_tgas(5),
            GasWeight::default(),
            YIELD_REGISTER,
        );

        // load the ID created by the promise_yield_create
        let id: CryptoHash = env::read_register(YIELD_REGISTER)
            .expect("read_register failed")
            .try_into()
            .expect("conversion to CryptoHash failed");

        // store the request
        self.requests.insert(id, prompt.clone());
        self.answers.insert(prompt, Response::Wait);

        // wait for the response
        env::promise_return(yield_promise);
    }

    // TODO: Add rules to decide who can call this function
    pub fn respond(&mut self, request_id: CryptoHash, response: String) {
        // insert answer
        let prompt = self.requests.get(&request_id).expect("Request not found");
        self.answers
            .insert(prompt.to_owned(), Response::Answer(response.clone()));

        // remove request
        self.requests.remove(&request_id);

        // resume computation
        env::promise_yield_resume(&request_id, &serde_json::to_vec(&response).unwrap());
    }

    #[private]
    pub fn return_external_response(
        &mut self,
        prompt: Prompt,
        #[callback_result] response: Result<String, PromiseError>,
    ) -> Response {
        match response {
            Ok(answer) => {
                self.answers.remove(&prompt);
                Response::Answer(answer)
            }
            Err(_) => {
                self.answers.insert(prompt, Response::Wait);
                Response::Wait
            }
        }
    }

    #[private]
    pub fn return_existing_response(
        &mut self,
        prompt: Prompt,
    ) -> Response {
        let answer = self.answers.get(&prompt).expect("Answer not found").clone();
        self.answers.remove(&prompt);
        answer
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
}
