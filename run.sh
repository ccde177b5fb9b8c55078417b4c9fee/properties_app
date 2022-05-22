#!/bin/bash
pushd frontend
trunk build --public-url /assets/ --release
tailwindcss -o ../dist/tailwind.css
popd 


export $(cat server/.env | xargs)

cargo run --bin server --release
