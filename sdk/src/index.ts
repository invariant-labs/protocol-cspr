import {
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  Keys,
  RuntimeArgs,
} from "casper-js-sdk";
import { FUNDED_KEYS } from "casper-node-launcher-js";
import { Client } from "./client";
import { getWasm } from "./utils";

const main = async () => {
  console.log("Init SDK!");

  const users = FUNDED_KEYS.map((k) => k.private).map((key) =>
    Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519)
  );
  const ALICE = users[0];
  const BOB = users[1];
  const CHARLIE = users[2];

  const NETWORK_URL = "http://127.0.0.1:7777/rpc";
  const NETWORK_NAME = "casper-net-1";

  const client = new Client(NETWORK_URL, NETWORK_NAME);
  const casperClient = new CasperServiceByJsonRPC(NETWORK_URL);
  const accountInfo = await client.getAccountInfo(ALICE.publicKey);
  const initialAssociatedKeys = accountInfo.associatedKeys;

  const newCasperClient = new CasperClient(NETWORK_URL);
  const contract = new Contracts.Contract(newCasperClient);

  const wasm = getWasm("invariant");
  const runtimeArguments = RuntimeArgs.fromMap({
    fee: CLValueBuilder.u256(100_000_000),
  });
  const deploy = contract.install(
    wasm,
    runtimeArguments,
    "10000000000",
    ALICE.publicKey,
    "casper-net-1",
    [ALICE]
  );
  console.log(await newCasperClient.putDeploy(deploy));

  process.exit(0);
};

main();
