import { CLPublicKey } from "casper-js-sdk";
import { Client } from "./client";
import { KEYS_ALGO, KEYS_PATH, NETWORK_NAME, NODE_ADDRESS } from "./consts";
import {
  createAccountKeys,
  getAccountInfo,
  getWasm,
  parseAccountKeys,
  sleep,
} from "./utils";

const main = async () => {
  console.log("Init SDK!");

  const accountAddress = createAccountKeys();
  const parsedAddress = parseAccountKeys(KEYS_PATH, KEYS_ALGO);
  const client = new Client(NODE_ADDRESS, NETWORK_NAME);
  console.log(client);

  await sleep(10000);

  const accountInfo = await getAccountInfo(
    NODE_ADDRESS,
    CLPublicKey.fromHex(accountAddress)
  );

  console.log(accountInfo);

  const invariantWasm = getWasm("invariant");
  const contractName = "Invariant";

  const installDeploy = client.install(
    contractName,
    "10000000000000",
    // CLPublicKey.fromHex(accountAddress),
    parsedAddress.publicKey,
    invariantWasm,
    [parsedAddress]
  );

  console.log(`... contract installation deploy`);
  console.log(installDeploy);
  const installDeployHash = await installDeploy.send(NODE_ADDRESS);
  console.log(`... contract installation deployHash: ${installDeployHash}`);

  //   await getDeploy(NODE_ADDRESS, installDeployHash);

  //   const accountInfos = await getAccountInfo(
  //     NODE_ADDRESS,
  //     CONTRACT_OWNER.publicKey
  //   );

  //   console.log(`... Account Info: `);
  //   console.log(JSON.stringify(accountInfos, null, 2));

  //   const paymentsContractHash = await getAccountNamedKeyValue(
  //     accountInfos,
  //     `${contractName}_contract_hash`
  //   );

  //   const paymentsContractPackageHash = await getAccountNamedKeyValue(
  //     accountInfos,
  //     `${contractName}_contract_package_hash`
  //   );

  //   console.log(
  //     `... Payments contract installed successfully. ${paymentsContractHash}`
  //   );
  //   console.log(`... Contract hash:         ${paymentsContractHash}`);
  //   console.log(`... Contract package hash: ${paymentsContractPackageHash}`);
  //   console.log(
  //     `----------------------------------------------------------------------------------------------------`
  //   );
  process.exit(0);
};

main();
