import {
  CLPublicKey,
  CLValueBuilder,
  CasperClient,
  Contracts,
  DeployUtil,
  Keys,
  RuntimeArgs,
} from "casper-js-sdk";
import { getWasm } from "./utils";

export class Invariant {
  casperClient: CasperClient;
  contract: Contracts.Contract;

  constructor(public nodeAddress: string, public networkName: string) {
    this.casperClient = new CasperClient(nodeAddress);
    this.contract = new Contracts.Contract(this.casperClient);
  }

  async deploy(signer: Keys.AsymmetricKey): Promise<string> {
    const wasm = getWasm("invariant");

    const runtimeArguments = RuntimeArgs.fromMap({
      fee: CLValueBuilder.u256(100_000_000),
    });

    const deploy = this.install(
      wasm,
      runtimeArguments,
      "10000000000",
      signer.publicKey,
      "casper-net-1",
      [signer]
    );

    const txHash = await this.casperClient.putDeploy(deploy);

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
}
