import { BigNumber } from "@ethersproject/bignumber";
// import {
//   getSqrtPriceDenominator,
//   getSqrtPriceScale,
// } from "invariant-cspr-wasm";
import { ALICE, NETWORK_NAME, NETWORK_URL } from "./consts";
import { Invariant } from "./invariant";
import { getDeploy, sleep } from "./utils";
const main = async () => {
  const wasm = require("invariant-cspr-wasm");

  console.log("Init SDK!");
  // {
  //   const sqrtPriceScale = getSqrtPriceScale();
  //   const sqrtPriceDenominator = getSqrtPriceDenominator();
  //   // const amount: TokenAmount = { v: 100000000n };
  //   console.log(sqrtPriceScale, sqrtPriceDenominator);
  // }
  {
    const invariant = new Invariant(NETWORK_URL, NETWORK_NAME);
    const aliceBalance: BigNumber =
      await invariant.casperClient.balanceOfByPublicKey(ALICE.publicKey);
    console.log(aliceBalance.toBigInt());

    const txHash = await invariant.deploy(ALICE);

    const deploy = await getDeploy(NETWORK_URL, txHash);
    console.log(deploy);

    await sleep(2000);

    const invtHash = await invariant.getContractHash(
      NETWORK_URL,
      ALICE,
      "invariant"
    );

    invariant.contract.setContractHash(invtHash);

    await sleep(1000);

    // const fetchedConfig = await invariant.contract.queryContractData([
    //   "config",
    // ]);
    // const args = RuntimeArgs.fromMap({});
    // const query = invariant.contract.callEntrypoint(
    //   "get_protocol_fee",
    //   args,
    //   ALICE.publicKey,
    //   NETWORK_NAME,
    //   "1000000000", // 1 CSPR (10^9 Motes)
    //   [ALICE]
    // );
    // console.log(await invariant.casperClient.putDeploy(query));
  }
  process.exit(0);
};

main();
