/* eslint-disable camelcase */
import { Some } from '@casperlabs/ts-results'
import { BigNumber } from '@ethersproject/bignumber'
import {
  CLByteArray,
  CLValueBuilder,
  CasperClient,
  Contracts,
  Keys,
  RuntimeArgs,
  decodeBase16
} from 'casper-js-sdk'
import { ALLOWANCES, BALANCES, DEFAULT_PAYMENT_AMOUNT, ERC20_CONTRACT_NAME } from './consts'
import { hash, hexToBytes } from './parser'
import { Key, Network } from './schema'
import { extractContractHash, extractContractPackageHash, getDeploymentData, sendTx } from './utils'

export class Erc20 {
  client: CasperClient
  contract: Contracts.Contract
  paymentAmount: bigint
  network: Network

  private constructor(
    client: CasperClient,
    network: Network,
    contractHash: string,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ) {
    this.client = client
    this.network = network
    this.contract = new Contracts.Contract(this.client)
    this.contract.setContractHash(contractHash)
    this.paymentAmount = paymentAmount
  }

  static async deploy(
    client: CasperClient,
    network: Network,
    deployer: Keys.AsymmetricKey,
    namedKeysName: string,
    initialSupply: bigint = 0n,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 9n,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ): Promise<[string, string]> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(ERC20_CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(namedKeysName),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      initial_supply: CLValueBuilder.option(Some(CLValueBuilder.u256(Number(initialSupply)))),
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

    await client.nodeClient.deploy(signedDeploy)

    const deploymentResult = await client.nodeClient.waitForDeploy(signedDeploy, 100000)

    if (deploymentResult.execution_results[0].result.Failure) {
      throw new Error(
        deploymentResult.execution_results[0].result.Failure.error_message?.toString()
      )
    }

    const stateRootHash = await client.nodeClient.getStateRootHash()
    const { Account } = await client.nodeClient.getBlockState(
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

    const { ContractPackage } = await client.nodeClient.getBlockState(
      stateRootHash,
      contractPackageHash,
      []
    )

    if (!ContractPackage) {
      throw new Error('Contract package not found in block state')
    }

    return [extractContractHash(contractPackageHash), extractContractPackageHash(ContractPackage)]
  }

  static async load(client: CasperClient, network: Network, contractHash: string) {
    return new Erc20(client, network, 'hash-' + contractHash)
  }

  setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async name(): Promise<string> {
    const response = await this.contract.queryContractDictionary('state', hash('name'))

    return response.data
  }

  async symbol(): Promise<string> {
    const response = await this.contract.queryContractDictionary('state', hash('symbol'))

    return response.data
  }

  async decimals(): Promise<bigint> {
    const response = await this.contract.queryContractDictionary('state', hash('decimals'))

    return BigInt(response.data)
  }

  async balanceOf(addressHash: Key, address: string): Promise<bigint> {
    const balanceKey = new Uint8Array([...BALANCES, addressHash, ...hexToBytes(address)])

    try {
      const response = await this.contract.queryContractDictionary('state', hash(balanceKey))
      return BigInt(response.data._hex)
    } catch (e) {
      return 0n
    }
  }

  async allowance(
    ownerHash: Key,
    owner: string,
    spenderHash: Key,
    spender: string
  ): Promise<bigint> {
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

  async approve(signer: Keys.AsymmetricKey, spenderHash: Key, spender: string, amount: bigint) {
    const spenderBytes = new Uint8Array([spenderHash, ...decodeBase16(spender)])
    const spenderKey = new CLByteArray(spenderBytes)

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'approve',
      {
        spender: spenderKey,
        amount: CLValueBuilder.u256(BigNumber.from(amount))
      }
    )
  }

  async transfer(
    signer: Keys.AsymmetricKey,
    recipientHash: Key,
    recipient: string,
    amount: bigint
  ) {
    const recipientBytes = new Uint8Array([recipientHash, ...decodeBase16(recipient)])
    const recipientKey = new CLByteArray(recipientBytes)

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'transfer',
      {
        recipient: recipientKey,
        amount: CLValueBuilder.u256(BigNumber.from(amount))
      }
    )
  }

  async mint(signer: Keys.AsymmetricKey, addressHash: Key, address: string, amount: bigint) {
    const addressBytes = new Uint8Array([addressHash, ...decodeBase16(address)])
    const addressKey = new CLByteArray(addressBytes)

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'mint',
      {
        address: addressKey,
        amount: CLValueBuilder.u256(BigNumber.from(amount))
      }
    )
  }

  async transferFrom(
    signer: Keys.AsymmetricKey,
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
      this.client.nodeClient,
      this.paymentAmount,
      signer,
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
