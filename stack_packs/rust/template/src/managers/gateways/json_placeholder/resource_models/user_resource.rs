use serde::Deserialize;

// Resource models: the JSON body deserialized as it is, mutable, with every field optional so that anything the API
// sends deserializes and the validator can say what is wrong with it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AddressResource {
    pub street: Option<String>,
    pub suite: Option<String>,
    pub city: Option<String>,
    pub zipcode: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CompanyResource {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UserResource {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<AddressResource>,
    pub company: Option<CompanyResource>,
}
