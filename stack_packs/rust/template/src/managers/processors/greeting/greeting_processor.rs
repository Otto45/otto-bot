use crate::managers::models::greeting::Greeting;

pub struct GreetingProcessor {
    greeting_prefix: String,
}

impl GreetingProcessor {
    pub fn new(greeting_prefix: String) -> Self {
        GreetingProcessor { greeting_prefix }
    }

    pub fn greet(&self, name: &str) -> Greeting {
        Greeting {
            message: format!("{}, {name}!", self.greeting_prefix),
        }
    }
}
