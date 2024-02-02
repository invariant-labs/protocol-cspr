import {
  CLValueBuilder,
  CasperClient,
  Contracts,
  RuntimeArgs,
} from "casper-js-sdk";
import { ALICE, NETWORK_NAME, NETWORK_URL } from "./consts";
import { Invariant } from "./invariant";
import { getWasm } from "./utils";

const main = async () => {
  console.log("Init SDK!");

  const casperClient = new CasperClient(NETWORK_URL);
  const contract = new Contracts.Contract(casperClient);

  const wasm = getWasm("invariant");
  const runtimeArguments = RuntimeArgs.fromMap({
    fee: CLValueBuilder.u256(100_000_000),
  });

  const deploy = contract.install(
    wasm,
    runtimeArguments,
    "10000000000",
    ALICE.publicKey,
    NETWORK_NAME,
    [ALICE]
  );

  const hash = await casperClient.putDeploy(deploy);
  // console.log("Tx hash: ", hash);
  // console.log("");

  {
    const invariant = new Invariant(NETWORK_URL, NETWORK_NAME);

    await invariant.deploy(ALICE);

    const contractHash = `hash-${invariant.contract.contractHash}`;
    invariant.contract.setContractHash(contractHash);

    const args = RuntimeArgs.fromMap({});
    const query = invariant.contract.callEntrypoint(
      "get_protocol_fee",
      args,
      ALICE.publicKey,
      NETWORK_NAME,
      "1000000000", // 1 CSPR (10^9 Motes)
      [ALICE]
    );
    console.log(await invariant.casperClient.putDeploy(query));
  }
  process.exit(0);
};

main();
