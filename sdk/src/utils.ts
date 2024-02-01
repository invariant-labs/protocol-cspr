import {
  CLPublicKey,
  CasperClient,
  CasperServiceByJsonRPC,
  Keys,
} from "casper-js-sdk";
import fs from "fs";
import path from "path";

export const sleep = (ms: number) => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

export const getWasm = (fileName: "invariant" | "token"): Uint8Array => {
  return new Uint8Array(
    fs.readFileSync(`./src/contracts/${fileName}.wasm`, null).buffer
  );
};

export const getDeploy = async (nodeURL: string, deployHash: string) => {
  const client = new CasperClient(nodeURL);
  let i = 300;
  while (i !== 0) {
    const [deploy, raw] = await client.getDeploy(deployHash);
    if (raw.execution_results.length !== 0) {
      // @ts-ignore
      if (raw.execution_results[0].result.Success) {
        return deploy;
      } else {
        // @ts-ignore
        throw Error(
          "Contract execution: " +
            // @ts-ignore
            raw.execution_results[0].result.Failure.error_message
        );
      }
    } else {
      i--;
      await sleep(1000);
      continue;
    }
  }
  throw Error("Timeout after " + i + "s. Something's wrong");
};

export const getAccountInfo: any = async (
  nodeAddress: string,
  publicKey: CLPublicKey
) => {
  const client = new CasperServiceByJsonRPC(nodeAddress);
  const stateRootHash = await client.getStateRootHash();
  const accountHash = publicKey.toAccountHashStr();
  const blockState = await client.getBlockState(stateRootHash, accountHash, []);
  return blockState.Account;
};

/**
 * Returns a value under an on-chain account's storage.
 * @param accountInfo - On-chain account's info.
 * @param namedKey - A named key associated with an on-chain account.
 */
export const getAccountNamedKeyValue = (accountInfo: any, namedKey: string) => {
  const found = accountInfo.namedKeys.find((i: any) => i.name === namedKey);
  if (found) {
    return found.key;
  }
  return undefined;
};

export const printHeader = (text: string) => {
  console.log(`******************************************`);
  console.log(`* ${text} *`);
  console.log(`******************************************`);
};

export const parseAccountKeys = (
  keys_path: string,
  algo: string
): Keys.AsymmetricKey => {
  let ACCOUNT_KEYS;
  if (algo == "ed25519") {
    ACCOUNT_KEYS = Keys.Ed25519.loadKeyPairFromPrivateFile(
      `${keys_path}/public_key.pem`
    );
  } else if (algo == "secp256K1") {
    ACCOUNT_KEYS = Keys.Secp256K1.loadKeyPairFromPrivateFile(
      `${keys_path}/public_key.pem`
    );
  } else {
    console.log("Invalid keys crypto algorithm provided");
    process.exit(1);
  }

  return ACCOUNT_KEYS;
};

export const createAccountKeys = () => {
  // Generating keys
  const edKeyPair = Keys.Ed25519.new();
  const { publicKey, privateKey } = edKeyPair;

  // Create a hexadecimal representation of the public key
  const accountAddress = publicKey.toHex();

  // Get the account hash (Uint8Array) from the public key
  const accountHash = publicKey.toAccountHash();

  // Store keys as PEM files
  const publicKeyInPem = edKeyPair.exportPublicKeyInPem();
  const privateKeyInPem = edKeyPair.exportPrivateKeyInPem();

  const folder = path.join("./", "casper_keys");

  if (!fs.existsSync(folder)) {
    const tempDir = fs.mkdirSync(folder);
  }

  fs.writeFileSync(folder + "/public_key.pem", publicKeyInPem);
  fs.writeFileSync(folder + "/private_key.pem", privateKeyInPem);

  return accountAddress;
};
