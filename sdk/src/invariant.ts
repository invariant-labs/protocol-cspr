import {
  CLPublicKey,
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  DeployUtil,
  Keys,
  RuntimeArgs,
} from "casper-js-sdk";
import { getAccountInfo, getWasm, sleep } from "./utils";

export class Invariant {
  rpc: CasperServiceByJsonRPC;
  casperClient: CasperClient;
  contract: Contracts.Contract;

  constructor(public nodeAddress: string, public networkName: string) {
    this.rpc = new CasperServiceByJsonRPC(nodeAddress);
    this.casperClient = new CasperClient(nodeAddress);
    this.contract = new Contracts.Contract(this.casperClient);
  }

  async deploy(signer: Keys.AsymmetricKey): Promise<string> {
    const wasm = getWasm("invariant");

    const protocolFeeBytes = new Uint8Array([
      10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    const runtimeArguments = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string("invariant"),
      odra_cfg_allow_key_override: CLValueBuilder.bool(false),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string("init"),
      protocol_fee: CLValueBuilder.byteArray(protocolFeeBytes),
    });

    const deploy = this.install(
      wasm,
      runtimeArguments,
      "10000000000000",
      signer.publicKey,
      "casper-net-1",
      [signer]
    );

    await this.rpc.deploy(deploy);
    await sleep(2500);
    await this.rpc.waitForDeploy(deploy, 100000);

    const txHash = await this.casperClient.putDeploy(deploy);
    await sleep(2500);
    return txHash;
  }

  install(
    wasm: Uint8Array,
    args: RuntimeArgs,
    paymentAmount: string,
    sender: CLPublicKey,
    chainName: string,
    signingKeys: Keys.AsymmetricKey[] = []
  ) {
    const deploy = DeployUtil.makeDeploy(
      new DeployUtil.DeployParams(sender, chainName),
      DeployUtil.ExecutableDeployItem.newModuleBytes(wasm, args),
      DeployUtil.standardPayment(paymentAmount)
    );

    const signedDeploy = deploy.sign(signingKeys);

    return signedDeploy;
  }

  async getContractHash(
    network: string,
    signer: Keys.AsymmetricKey
  ): Promise<string> {
    const accountInfo = await getAccountInfo(network, signer.publicKey);
    console.log(accountInfo);

    const invtHash = accountInfo!.namedKeys.find(
      (i: any) => i.name === "invariant"
    )?.key;
    return invtHash;
  }
}
