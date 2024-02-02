import {
  CLValueBuilder,
  CasperClient,
  Contracts,
  RuntimeArgs,
} from "casper-js-sdk";
import { Client } from "./client";
import { KEYS_ALGO, KEYS_PATH, NETWORK_NAME, NODE_ADDRESS } from "./consts";
import { createAccountKeys, getWasm, parseAccountKeys, sleep } from "./utils";

const main = async () => {
  console.log("Init SDK!");

  const accountAddress = createAccountKeys();
  const parsedAddress = parseAccountKeys(KEYS_PATH, KEYS_ALGO);
  const client = new Client(NODE_ADDRESS, NETWORK_NAME);
  console.log(client);

  await sleep(1000);

  //   const accountInfo = await getAccountInfo(
  //     NODE_ADDRESS,
  //     CLPublicKey.fromHex(accountAddress)
  //   );

  const invariantWasm = getWasm("invariant");
  const contractName = "Invariant";

  {
    const casperClient = new CasperClient(NODE_ADDRESS + "/rpc");
    const contract = new Contracts.Contract(casperClient);
    const runtimeArguments = RuntimeArgs.fromMap({
      fee: CLValueBuilder.string("100000000000"),
    });
    const deploy = contract.install(
      invariantWasm,
      runtimeArguments,
      "10000000000",
      parsedAddress.publicKey,
      NETWORK_NAME,
      [parsedAddress]
    );
    console.log(await casperClient.putDeploy(deploy));
  }

  await sleep(1000);

  {
    const installDeploy = client.install(
      contractName,
      "100000000000",
      // CLPublicKey.fromHex(accountAddress),
      parsedAddress.publicKey,
      invariantWasm,
      [parsedAddress]
    );

    console.log(`... contract installation deploy`);
    console.log(installDeploy);
    const installDeployHash = await installDeploy.send(NODE_ADDRESS);
    console.log(`... contract installation deployHash: ${installDeployHash}`);
  }

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
