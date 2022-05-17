#!/bin/bash
pushd frontend
trunk build --public-url /assets/ --release
popd 

tailwindcss -o dist/tailwind.css

cargo run --bin server --release
