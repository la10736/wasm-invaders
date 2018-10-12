<meta charset="utf-8"/>

# `wasm-invaders`

Space Invaders retro' game powered by wasm and
[rs8080](https://github.com/la10736/rs8080) (a 8080 emulator writen
in Rust).

![Space Invaders Screenshoot](resources/si.png)

## Requirment

### The Rust Toolchain

You will need the standard Rust toolchain, including rustup, rustc, and cargo.

[Follow these instructions to install the Rust toolchain](https://www.rust-lang.org/en-US/install.html).

The Rust and WebAssembly experience is riding the Rust release trains to
stable! That means we don't require any experimental feature flags.
However, we do require Rust 1.30 or newer, and currently the Rust
stable branch is at 1.29. Therefore, use the beta branch until the
release trains roll over on 2018-10-25:

```
rustup default beta
```

or nightly

```
rustup default nightly
```

### `wasm-pack`
`wasm-pack` is your one-stop shop for building, testing, and publishing
Rust-generated WebAssembly.

[Get `wasm-pack` here!](https://rustwasm.github.io/wasm-pack/installer/)


### `npm`
`npm` is a package manager for JavaScript. We will use it to install and
run a JavaScript bundler and development server. At the end of the
tutorial, we will publish our compiled .wasm to the `npm` registry.

[Follow these instructions to install npm](https://www.npmjs.com/get-npm).

If you already have npm installed, make sure it is up to date with this command:

```
npm install npm@latest -g
```

## Compile

To compile use
```
./compile.sh
```

## Prepare node environment

Just use `./init.sh` to link this module in your node envirorment. You need
it just the first time, after that you can forget it.


## Serve

By `./start.s` you can start a web servel on http://localhost:8080 to
serve the app.

