# Cancun EELS conformance tests

The native `spectests` runner executes the Cancun state-test fixtures published
by the Ethereum Execution Layer Specifications project. The suite is pinned to
release `tests@v20.0.1`; its metadata is recorded in
`spectests/fixtures-release.json`.

Download and verify the pinned fixture archive before extracting the fixture
index and Cancun state tests:

```bash
curl --fail --location --retry 3 \
  --output fixtures.tar.gz \
  'https://github.com/ethereum/execution-specs/releases/download/tests%40v20.0.1/fixtures.tar.gz'
echo '3586193db06d4d5745d5e90b3c3008c2255a4e19ccd8f11a3ce887aec8c0b17c  fixtures.tar.gz' \
  | sha256sum --check --strict
mkdir -p .cache/eels-fixtures/tests-v20.0.1
tar --extract --gzip --file fixtures.tar.gz \
  --directory .cache/eels-fixtures/tests-v20.0.1 \
  --wildcards \
  fixtures/.meta/index.json \
  'fixtures/state_tests/for_cancun/*'
```

Run the complete pinned Cancun target in release mode:

```bash
cargo run --release -p spectests -- \
  --fixtures .cache/eels-fixtures/tests-v20.0.1/fixtures \
  --target Cancun
```

Without `--case` filters, the runner verifies the release index and requires
all 11,554 Cancun cases. Any missing or failed case fails the command, making
this invocation the full Cancun conformance gate used in CI. For focused
debugging, repeat `--case <fixture-id>` and optionally set `--jobs <count>`.

The older test repositories and debugging workflow remain available through
the [legacy `jsontests` documentation](ethtests.md).
