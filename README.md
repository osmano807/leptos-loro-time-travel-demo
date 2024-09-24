# Loro Time Travel Demo usin Leptos

This demo is intended to demonstrate [Loro](https://github.com/loro-dev/loro)'s high performance and time travel capabilities while using [Leptos](https://github.com/leptos-rs/leptos) as the frontend framework.

https://github.com/user-attachments/assets/d3eefe71-ee27-4657-be73-212852b091c0

This is a CSR example, for demonstration purposes, the `LoroDoc` snapshot is embedded into the frontend WASM. In a real application, the snapshot would be loaded from a server endpoint, see the `main` branch for an example of how to load the snapshot from a server endpoint.

The editing trace is from [josephg/editing-traces](https://github.com/josephg/editing-traces).

# Usage

1. Install rust and trunk
2. Run `trunk serve --port 8000`
3. Open `http://localhost:8000` in your browser

# Deploying

1. Run `trunk build --release`
2. Copy the contents of `dist` to your web server
