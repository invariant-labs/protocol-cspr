/* eslint-disable camelcase */
import { Some } from '@casperlabs/ts-results'
import {
  CLPublicKey,
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  DeployUtil,
  Keys,
  RuntimeArgs
} from 'casper-js-sdk'
import { getWasm, sleep } from './utils'

export class Erc20 {
  rpc: CasperServiceByJsonRPC
  casperClient: CasperClient
  contract: Contracts.Contract

  constructor(public nodeAddress: string, public networkName: string) {
    this.rpc = new CasperServiceByJsonRPC(nodeAddress)
    this.casperClient = new CasperClient(nodeAddress)
    this.contract = new Contracts.Contract(this.casperClient)
  }

  async deploy(
    signer: Keys.AsymmetricKey,
    symbol: string,
    name: string,
    decimals: bigint,
    initial_supply: bigint
  ): Promise<string> {
    const wasm = getWasm('erc20')

    const runtimeArguments = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string('erc20'),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      symbol: CLValueBuilder.string(symbol),
      name: CLValueBuilder.string(name),
      decimals: CLValueBuilder.u8(Number(decimals)),
      initial_supply: CLValueBuilder.option(Some(CLValueBuilder.u256(Number(initial_supply))))
    })

    const deploy = this.install(
      wasm,
      runtimeArguments,
      '10000000000000',
      signer.publicKey,
      'casper-net-1',
      [signer]
    )

    await this.rpc.deploy(deploy)

    await sleep(2500)
    const deployResult = await this.rpc.waitForDeploy(deploy, 100000)

    return deployResult.deploy.hash
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
    )

    const signedDeploy = deploy.sign(signingKeys)

    return signedDeploy
  }

  async getContractHash(
    network: string,
    signer: Keys.AsymmetricKey,
    contractName: string
  ): Promise<string> {
    const stateRootHash = await this.rpc.getStateRootHash()
    const accountHash = signer.publicKey.toAccountHashStr()
    const { Account } = await this.rpc.getBlockState(stateRootHash, accountHash, [])

    const hash = Account!.namedKeys.find((i: any) => i.name === contractName)?.key

    if (!hash) {
      return 'Contract not found!'
    }

    return hash
  }
}
