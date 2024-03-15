<div align="center">
    <h1>⚡Invariant protocol⚡</h1>
    <p>
        | <a href="https://docs.invariant.app/docs/casper">DOCS 📚</a> |
        <a href="https://invariant.app/math-spec-cspr.pdf">MATH SPEC 📄</a> |
        <a href="https://discord.gg/VzS3C9wR">DISCORD 🌐</a> |
    </p>
</div>

Invariant protocol is an AMM built on [Casper Network](https://casper.network/), leveraging high capital efficiency and the ability to list markets in a permissionless manner. At the core of the DEX is the Concentrated Liquidity mechanism, designed to handle tokens compatible with the [Erc20 standard](https://github.com/odradev/odra/blob/9b753cc23668709eddddcf7f078cdd60861592fb/modules/src/erc20.rs). The protocol is structured around a single contract architecture.

## Usage

It's recommend to install:

- [cargo-odra](https://github.com/odradev/cargo-odra)
- [wasm-strip](https://github.com/WebAssembly/wabt)

To install `wasm-strip` using apt, run the following command:

```bash
sudo apt install wabt
```

Additionally, add the wasm32-unknown-unknown target by running:

```bash
rustup target add wasm32-unknown-unknown
```

### Build

```
$ cargo odra build
```

To build a wasm file, you need to pass the -b parameter.
The result files will be placed in `${project-root}/wasm` directory.

```
$ cargo odra build -b casper
```

### Test

To run tests, choose between the `MockVM` and `Casper backend` using the following commands:

#### Run tests on MockVM

```bash
cargo odra test
```

#### Run tests on Casper backend

To test actual wasm files against a backend,
you need to specify the backend passing -b argument to `cargo-odra`.

```bash
cargo odra test -b casper
```

### Typescript SDK

Utilize the Invariant SDK from the npm registry by including it as a dependency. Alternatively, for a customized approach, build the SDK on your own using the commands provided below:

To begin, navigate to the directory dedicated to the [SDK](https://github.com/invariant-labs/protocol-cspr/tree/master/sdk)

- Build SDK
```bash
./build.sh
```

- Run e2e tests
```bash
./test.sh
```