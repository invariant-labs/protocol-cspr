import { Client } from "./client";
import { NETWORK_NAME, NODE_ADDRESS } from "./consts";
import { getWasm, parseAccountKeys } from "./utils";

const main = async () => {
  console.log("Init SDK!");

  const OWNER_KEYS_PATH = "./casper_keys";
  const KEYS_ALGO = "ed25519";

  const CONTRACT_OWNER = parseAccountKeys(OWNER_KEYS_PATH, KEYS_ALGO);

  const client = new Client(NODE_ADDRESS, NETWORK_NAME);
  console.log(client);

  const invariantWasm = getWasm("invariant");
  const contractName = "Invariant";

  const installDeploy = client.install(
    contractName,
    "10000000000000",
    CONTRACT_OWNER.publicKey,
    invariantWasm,
    [CONTRACT_OWNER]
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
};

main();
