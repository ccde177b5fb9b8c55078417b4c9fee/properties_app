-- Add up migration script here
CREATE TABLE properties(
    id SERIAL PRIMARY KEY NOT NULL,
    name VARCHAR(256) NOT NULL,
    location int NOT NULL,
    area int NOT NULL,
    property_type int NOT NULL,
    wc int NOT NULL,
    floor int NOT NULL,
    tothesea int NOT NULL,
    furniture boolean NOT NULL,
    appliances boolean NOT NULL,
    price int NOT NULL,
    posting_date TIMESTAMPTZ DEFAULT NOW(),
    gallery_location VARCHAR(256)
);
