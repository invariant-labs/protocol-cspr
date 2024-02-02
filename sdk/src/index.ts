import { BigNumber } from "@ethersproject/bignumber";
import { ALICE, NETWORK_NAME, NETWORK_URL } from "./consts";
import { Invariant } from "./invariant";
import { getAccountInfo, getDeploy } from "./utils";

const main = async () => {
  console.log("Init SDK!");

  {
    const invariant = new Invariant(NETWORK_URL, NETWORK_NAME);
    const aliceBalance: BigNumber =
      await invariant.casperClient.balanceOfByPublicKey(ALICE.publicKey);
    console.log(aliceBalance.toBigInt());

    const txHash = await invariant.deploy(ALICE);

    const deploy = await getDeploy(NETWORK_URL, txHash);
    console.log(deploy);

    const accountInfo = await getAccountInfo(NETWORK_URL, ALICE.publicKey);
    console.log(accountInfo);

    const invtHash = accountInfo!.namedKeys.find(
      (i: any) => i.name === "invariant"
    )?.key;

    invariant.contract.setContractHash(invtHash);

    // const args = RuntimeArgs.fromMap({});
    // const query = invariant.contract.callEntrypoint(
    //   "get_protocol_fee",
    //   args,
    //   ALICE.publicKey,
    //   NETWORK_NAME,
    //   "1000000000000", // 1 CSPR (10^9 Motes)
    //   [ALICE]
    // );
    // console.log(await invariant.casperClient.putDeploy(query));
  }
  process.exit(0);
};

main();
