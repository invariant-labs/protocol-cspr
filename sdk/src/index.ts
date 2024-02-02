import { BigNumber } from "@ethersproject/bignumber";
import {
  CLValueBuilder,
  CasperClient,
  Contracts,
  RuntimeArgs,
} from "casper-js-sdk";
import { ALICE, NETWORK_NAME, NETWORK_URL } from "./consts";
import { Invariant } from "./invariant";
import { getAccountInfo, getWasm } from "./utils";

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

    await invariant.deploy(ALICE);

    const accountInfo = await getAccountInfo(NETWORK_URL, ALICE.publicKey);
    console.log(accountInfo);

    const invtHash = accountInfo!.namedKeys.find(
      (i: any) => i.name === "erc20_token_contract"
    )?.key;

    const contractHash = `hash-${invariant.contract.contractHash}`;

    console.log(contractHash);
    invariant.contract.setContractHash(contractHash);

    // const args = RuntimeArgs.fromMap({});
    // const query = invariant.contract.callEntrypoint(
    //   "get_protocol_fee",
    //   args,
    //   ALICE.publicKey,
    //   NETWORK_NAME,
    //   "1000000000000000", // 1 CSPR (10^9 Motes)
    //   [ALICE]
    // );
    // console.log(await invariant.casperClient.putDeploy(query));
  }
  process.exit(0);
};

main();
