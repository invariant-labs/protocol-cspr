import { BigNumber } from "@ethersproject/bignumber";
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

  await casperClient.putDeploy(deploy);

  // ALICE.

  {
    const invariant = new Invariant(NETWORK_URL, NETWORK_NAME);
    const AliceBalance: BigNumber =
      await invariant.casperClient.balanceOfByPublicKey(ALICE.publicKey);
    console.log(AliceBalance.toBigInt());

    const txHash = await invariant.deploy(ALICE);

    // const deploy = await getDeploy(NETWORK_URL, txHash);
    // console.log(deploy);

    const invtHash = await invariant.getContractHash(NETWORK_URL, ALICE);
    invariant.contract.setContractHash(invtHash);

    // const args = RuntimeArgs.fromMap({});
    // const query = invariant.contract.callEntrypoint(
    //   "getProtocolFee",
    //   args,
    //   ALICE.publicKey,
    //   NETWORK_NAME,
    //   "100000000000", // 1 CSPR (10^9 Motes)
    //   [ALICE]
    // );
    // console.log(await invariant.casperClient.putDeploy(query));
  }
  process.exit(0);
};

main();
