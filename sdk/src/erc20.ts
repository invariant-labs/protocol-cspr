/* eslint-disable camelcase */
import { Some } from '@casperlabs/ts-results'
import { BigNumber } from '@ethersproject/bignumber'
import {
  CLByteArray,
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  Keys,
  RuntimeArgs,
  decodeBase16
} from 'casper-js-sdk'
import { ALLOWANCES, BALANCES, DEFAULT_PAYMENT_AMOUNT } from './consts'
import { Key, Network } from './enums'
import { getDeploymentData, hash, hexToBytes, sendTx } from './utils'

const CONTRACT_NAME = 'erc20'

export class Erc20 {
  client: CasperClient
  service: CasperServiceByJsonRPC
  contract: Contracts.Contract
  paymentAmount: bigint
  network: Network

  private constructor(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    contractHash: string,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ) {
    this.client = client
    this.service = service
    this.network = network
    this.contract = new Contracts.Contract(this.client)
    this.contract.setContractHash(contractHash)
    this.paymentAmount = paymentAmount
  }

  static async deploy(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    deployer: Keys.AsymmetricKey,
    namedKeysName: string,
    initial_supply: bigint = 0n,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ): Promise<[string, string]> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(namedKeysName),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      initial_supply: CLValueBuilder.option(Some(CLValueBuilder.u256(Number(initial_supply)))),
      name: CLValueBuilder.string(name),
      symbol: CLValueBuilder.string(symbol),
      decimals: CLValueBuilder.u8(Number(decimals))
    })

    const signedDeploy = contract.install(
      wasm,
      args,
      paymentAmount.toString(),
      deployer.publicKey,
      network.toString(),
      [deployer]
    )

    await service.deploy(signedDeploy)

    const deploymentResult = await service.waitForDeploy(signedDeploy, 100000)

    if (deploymentResult.execution_results[0].result.Failure) {
      throw new Error(
        deploymentResult.execution_results[0].result.Failure.error_message?.toString()
      )
    }

    const stateRootHash = await service.getStateRootHash()
    const { Account } = await service.getBlockState(
      stateRootHash,
      deployer.publicKey.toAccountHashStr(),
      []
    )

    if (!Account) {
      throw new Error('Account not found in block state')
    }

    const contractPackageHash = Account.namedKeys.find((i: any) => i.name === namedKeysName)?.key

    if (!contractPackageHash) {
      throw new Error('Contract package not found in account named keys')
    }

    const { ContractPackage } = await service.getBlockState(stateRootHash, contractPackageHash, [])

    if (!ContractPackage) {
      throw new Error('Contract package not found in block state')
    }

    return [
      contractPackageHash.replace('hash-', ''),
      ContractPackage.versions[0].contractHash.replace('contract-', '')
    ]
  }

  static async load(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    contractHash: string
  ) {
    return new Erc20(client, service, network, 'hash-' + contractHash)
  }

  setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async name() {
    const response = await this.contract.queryContractDictionary('state', hash('name'))

    return response.data
  }

  async symbol() {
    const response = await this.contract.queryContractDictionary('state', hash('symbol'))

    return response.data
  }

  async decimals() {
    const response = await this.contract.queryContractDictionary('state', hash('decimals'))

    return BigInt(response.data)
  }

  async balanceOf(addressHash: Key, address: string) {
    const balanceKey = new Uint8Array([...BALANCES, addressHash, ...hexToBytes(address)])

    try {
      const response = await this.contract.queryContractDictionary('state', hash(balanceKey))
      return BigInt(response.data._hex)
    } catch (e) {
      return 0n
    }
  }

  async allowance(ownerHash: Key, owner: string, spenderHash: Key, spender: string) {
    const balanceKey = new Uint8Array([
      ...ALLOWANCES,
      ownerHash,
      ...hexToBytes(owner),
      spenderHash,
      ...hexToBytes(spender)
    ])

    const response = await this.contract.queryContractDictionary('state', hash(balanceKey))

    return BigInt(response.data._hex)
  }

  async approve(account: Keys.AsymmetricKey, spenderHash: Key, spender: string, amount: bigint) {
    const spenderBytes = new Uint8Array([spenderHash, ...decodeBase16(spender)])
    const spenderKey = new CLByteArray(spenderBytes)

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'approve',
      {
        spender: spenderKey,
        amount: CLValueBuilder.u256(BigNumber.from(amount))
      }
    )
  }

  async transfer(
    account: Keys.AsymmetricKey,
    recipientHash: Key,
    recipient: string,
    amount: bigint
  ) {
    const recipientBytes = new Uint8Array([recipientHash, ...decodeBase16(recipient)])
    const recipientKey = new CLByteArray(recipientBytes)

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'transfer',
      {
        recipient: recipientKey,
        amount: CLValueBuilder.u256(Number(amount))
      }
    )
  }

  async mint(account: Keys.AsymmetricKey, addressHash: Key, address: string, amount: bigint) {
    const addressBytes = new Uint8Array([addressHash, ...decodeBase16(address)])
    const addressKey = new CLByteArray(addressBytes)

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'mint',
      {
        address: addressKey,
        amount: CLValueBuilder.u256(BigNumber.from(amount))
      }
    )
  }

  async transferFrom(
    account: Keys.AsymmetricKey,
    ownerHash: Key,
    owner: string,
    recipientHash: Key,
    recipient: string,
    amount: bigint
  ) {
    const ownerBytes = new Uint8Array([ownerHash, ...decodeBase16(owner)])
    const ownerKey = new CLByteArray(ownerBytes)
    const recipientBytes = new Uint8Array([recipientHash, ...decodeBase16(recipient)])
    const recipientKey = new CLByteArray(recipientBytes)

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'transfer_from',
      {
        owner: ownerKey,
        recipient: recipientKey,
        amount: CLValueBuilder.u256(BigNumber.from(amount))
      }
    )
  }
}
