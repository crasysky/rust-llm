mod api;

use api::*;

pub type ClientResponse = String;

pub trait ClientApi {
    fn response(&self, dialog: Vec<Message>) -> Result<ClientResponse, String>;
}

