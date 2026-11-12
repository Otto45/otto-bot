-- migrate:up
CREATE TABLE customers (
    customer_id TEXT PRIMARY KEY,
    full_name TEXT,
    email_address TEXT
);

-- migrate:down
DROP TABLE customers;
