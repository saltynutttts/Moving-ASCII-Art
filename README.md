# ASCII Warp Tool

A Rust tool that converts images into animated ASCII art. Supports warping and distortion to create moving, fluid ASCII visuals rendered in the browser.

# Usage

Start a local server in a new terminal:

python -m http.server 8000

Run the Rust generator using an image:

cargo run --bin main

Run the Rust generator without an image and using perlin noise alone:

cargo run --bin NoImage

Open http://localhost:8000/ in your browser to view the animated ASCII output.

# Custom Images
Drop an image into the project folder, update the image filename in main.rs, then rerun the program.
