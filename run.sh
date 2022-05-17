#!/bin/bash
pushd frontend
trunk build --public-url /assets/
popd 

tailwindcss -o dist/tailwind.css

cargo run --bin server --release
