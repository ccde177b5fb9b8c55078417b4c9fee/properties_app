-- Add migration script here
CREATE TABLE properties(
    id SERIAL PRIMARY KEY,
    name CHAR(256),
    location int,
    area int,
    type int,
    wc int,
    floor int,
    tothesea int,
    furniture boolean,
    appliances boolean,
    posting_date DATE NOT NULL DEFAULT NOW()
);

    
