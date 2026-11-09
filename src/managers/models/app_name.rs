/// An app's name in each form a template's names are replaced with, from the words of the name it was given (e.g.
/// "Orders App": orders-app, orders_app, OrdersApp, ordersApp, ORDERS_APP, Orders App).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppName {
    pub kebab: String,
    pub snake: String,
    pub pascal: String,
    pub camel: String,
    pub upper: String,
    pub title: String,
}
