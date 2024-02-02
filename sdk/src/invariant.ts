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
import { getWasm, sleep } from "./utils";

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

    const bytes = new Uint8Array([
      10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    const runtimeArguments = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string("invariant"),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string("init"),
      fee: CLValueBuilder.u128(0),
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
    const deployResult = await this.rpc.waitForDeploy(deploy, 100000);
    console.log(deployResult.execution_results[0]);
    console.log(deployResult.deploy.hash);
    return deployResult.deploy.hash;
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
}
