import { BigNumber } from "@ethersproject/bignumber";
import { ALICE, NETWORK_NAME, NETWORK_URL } from "./consts";
import { Invariant } from "./invariant";
import { getAccountInfo } from "./utils";

const main = async () => {
  console.log("Init SDK!");

  {
    const invariant = new Invariant(NETWORK_URL, NETWORK_NAME);
    const aliceBalance: BigNumber =
      await invariant.casperClient.balanceOfByPublicKey(ALICE.publicKey);
    console.log(aliceBalance.toBigInt());

    await invariant.deploy(ALICE);

    const accountInfo = await getAccountInfo(NETWORK_URL, ALICE.publicKey);
    console.log(accountInfo);

    const invtHash = accountInfo!.namedKeys.find(
      (i: any) => i.name === "invariant"
    )?.key;

    // const contractHash = `hash-${invariant.contract.contractHash}`;

    // console.log(contractHash);
    // invariant.contract.setContractHash(contractHash);

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
