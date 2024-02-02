import { BigNumber, type BigNumberish } from "@ethersproject/bignumber";
import {
  CLAccountHash,
  CLPublicKey,
  CLValueBuilder,
  CasperServiceByJsonRPC,
  Contracts,
  DeployUtil,
  Keys,
  RuntimeArgs,
  StoredValue,
  encodeBase16,
} from "casper-js-sdk";
const { Contract } = Contracts;

type Defined<Value> = Exclude<Value, null | undefined>;
type Account = Defined<StoredValue["Account"]>;

export class Client {
  casperClient: CasperServiceByJsonRPC;

  constructor(public nodeAddress: string, public networkName: string) {
    this.casperClient = new CasperServiceByJsonRPC(nodeAddress);
  }

  async getAccountInfo(account: CLPublicKey | CLAccountHash): Promise<Account> {
    const accountHash =
      account instanceof CLPublicKey
        ? new CLAccountHash(account.toAccountHash())
        : account;
    const stateRootHash = await this.casperClient.getStateRootHash();
    const accountInfo = await this.casperClient.getBlockState(
      stateRootHash,
      `account-hash-${encodeBase16(accountHash.value())}`,
      []
    );

    if (!accountInfo.Account) {
      throw new Error(`Account ${accountHash} not found`);
    }

    return accountInfo.Account;
  }

  async waitForDeploy(deploy: DeployUtil.Deploy | string) {
    return this.casperClient.waitForDeploy(deploy);
  }

  private install(
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

  addAssociatedKeys(
    wasm: Uint8Array,
    args: AddAssociatedKeysArgs,
    paymentAmount: BigNumberish,
    sender: CLPublicKey,
    networkName?: string,
    signingKeys?: Keys.AsymmetricKey[]
  ): DeployUtil.Deploy {
    const runtimeArgs = RuntimeArgs.fromMap({
      keys: CLValueBuilder.list(
        args.keys.map((key) =>
          CLValueBuilder.tuple2([
            key.account instanceof CLPublicKey
              ? new CLAccountHash(key.account.toAccountHash())
              : key.account,
            CLValueBuilder.u8(key.weight),
          ])
        )
      ),
    });

    return this.install(
      wasm,
      runtimeArgs,
      BigNumber.from(paymentAmount).toString(),
      sender,
      networkName ?? this.networkName,
      signingKeys
    );
  }

  removeAssociatedKeys(
    wasm: Uint8Array,
    args: RemoveAssociatedKeysArgs,
    paymentAmount: BigNumberish,
    sender: CLPublicKey,
    networkName?: string,
    signingKeys?: Keys.AsymmetricKey[]
  ): DeployUtil.Deploy {
    const runtimeArgs = RuntimeArgs.fromMap({
      keys: CLValueBuilder.list(
        args.keys.map((key) =>
          key instanceof CLPublicKey
            ? new CLAccountHash(key.toAccountHash())
            : key
        )
      ),
    });

    return this.install(
      wasm,
      runtimeArgs,
      BigNumber.from(paymentAmount).toString(),
      sender,
      networkName ?? this.networkName,
      signingKeys
    );
  }

  updateAssociatedKeys(
    wasm: Uint8Array,
    args: UpdateAssociatedKeysArgs,
    paymentAmount: BigNumberish,
    sender: CLPublicKey,
    networkName?: string,
    signingKeys?: Keys.AsymmetricKey[]
  ): DeployUtil.Deploy {
    const runtimeArgs = RuntimeArgs.fromMap({
      keys: CLValueBuilder.list(
        args.keys.map((key) =>
          CLValueBuilder.tuple2([
            key.account instanceof CLPublicKey
              ? new CLAccountHash(key.account.toAccountHash())
              : key.account,
            CLValueBuilder.u8(key.weight),
          ])
        )
      ),
    });

    return this.install(
      wasm,
      runtimeArgs,
      BigNumber.from(paymentAmount).toString(),
      sender,
      networkName ?? this.networkName,
      signingKeys
    );
  }

  updateThreshold(
    wasm: Uint8Array,
    args: UpdateThresholdArgs,
    paymentAmount: BigNumberish,
    sender: CLPublicKey,
    networkName?: string,
    signingKeys?: Keys.AsymmetricKey[]
  ): DeployUtil.Deploy {
    const runtimeArgs = RuntimeArgs.fromMap({
      key_management_threshold: CLValueBuilder.u8(args.keyManagement),
      deploy_threshold: CLValueBuilder.u8(args.deployment),
    });

    return this.install(
      wasm,
      runtimeArgs,
      BigNumber.from(paymentAmount).toString(),
      sender,
      networkName ?? this.networkName,
      signingKeys
    );
  }
}

export interface AssociatedKey {
  account: CLPublicKey | CLAccountHash;
  weight: number;
}

export interface AddAssociatedKeysArgs {
  keys: AssociatedKey[];
}

export interface RemoveAssociatedKeysArgs {
  keys: AssociatedKey["account"][];
}

export interface UpdateAssociatedKeysArgs {
  keys: AssociatedKey[];
}

export interface UpdateThresholdArgs {
  deployment: number;
  keyManagement: number;
}
