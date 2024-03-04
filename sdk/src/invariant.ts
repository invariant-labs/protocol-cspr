/* eslint-disable camelcase */
import { BigNumber } from '@ethersproject/bignumber'
import {
  CLAccountHashBytesParser,
  CLBoolBytesParser,
  CLI32BytesParser,
  CLOptionBytesParser,
  CLOptionType,
  CLStringBytesParser,
  CLTypeBuilder,
  CLU128BytesParser,
  CLU256BytesParser,
  CLU32BytesParser,
  CLU64BytesParser,
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  Keys,
  RuntimeArgs
} from 'casper-js-sdk'
import { DEFAULT_PAYMENT_AMOUNT, TESTNET_NODE_URL } from './consts'
import { Network } from './network'
import { bytesToHex, getDeploymentData, hash, lowerCaseFirstLetter, sendTx, unwrap } from './utils'

const CONTRACT_NAME = 'invariant'

export class Invariant {
  client: CasperClient
  service: CasperServiceByJsonRPC
  contract: Contracts.Contract
  paymentAmount: bigint

  private constructor(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    contractHash: string,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ) {
    this.client = client
    this.service = service
    this.contract = new Contracts.Contract(this.client)
    this.contract.setContractHash(contractHash)
    this.paymentAmount = paymentAmount
  }

  static async deploy(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    deployer: Keys.AsymmetricKey,
    fee: bigint = 0n,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ): Promise<string> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(CONTRACT_NAME),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      fee: CLValueBuilder.u128(Number(fee))
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
      // Shows the required deploy fee in case of failure
      console.log('----------')
      for (const v of deploymentResult.execution_results[0].result.Failure!.effect.transforms) {
        console.log(v)
      }
      console.log('----------')

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

    const contractPackageHash = Account.namedKeys.find((i: any) => i.name === CONTRACT_NAME)?.key

    if (!contractPackageHash) {
      throw new Error('Contract package not found in account named keys')
    }

    const { ContractPackage } = await service.getBlockState(stateRootHash, contractPackageHash, [])

    if (!ContractPackage) {
      throw new Error('Contract package not found in block state')
    }

    return ContractPackage.versions[0].contractHash.replace('contract-', '')
  }

  static async load(client: CasperClient, service: CasperServiceByJsonRPC, contractHash: string) {
    return new Invariant(client, service, 'hash-' + contractHash)
  }

  async setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async addFeeTier(
    account: Keys.AsymmetricKey,
    network: Network,
    fee: bigint,
    tickSpacing: bigint
  ) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      network,
      'add_fee_tier',
      {
        fee: CLValueBuilder.u128(Number(fee)),
        tick_spacing: CLValueBuilder.u32(Number(tickSpacing))
      }
    )
  }

  async changeProtocolFee(account: Keys.AsymmetricKey, network: Network, protocolFee: bigint) {
    const txArgs = RuntimeArgs.fromMap({
      protocol_fee: CLValueBuilder.u128(BigNumber.from(protocolFee))
    })

    const deploy = this.contract.callEntrypoint(
      'change_protocol_fee',
      txArgs,
      account.publicKey,
      network,
      DEFAULT_PAYMENT_AMOUNT.toString(),
      [account]
    )

    deploy.sign([account])
    await deploy.send(TESTNET_NODE_URL)
    await this.service.deploy(deploy)
    return await this.service.waitForDeploy(deploy, 100000)
  }

  async getInvariantConfig() {
    const key = hash('config')
    const stateRootHash = await this.service.getStateRootHash()

    const response = await this.client.nodeClient.getDictionaryItemBytesByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key
    )

    const bytes = new Uint8Array(
      String(response)
        .match(/.{1,2}/g)!
        .map((byte: string) => parseInt(byte, 16))
    )

    const stringParser = new CLStringBytesParser()
    const u256Parser = new CLU256BytesParser()
    const addressParser = new CLAccountHashBytesParser()

    const { result: stringResult, remainder: stringRemainder } =
      stringParser.fromBytesWithRemainder(bytes)

    const structName = lowerCaseFirstLetter(unwrap(stringResult, 'Couldnt parse string'))

    // One additional byte is left on the beggining of the bytes remainder
    const updatedRemainder = stringRemainder!.slice(1, stringRemainder!.length)

    const { result: addressResult, remainder: addressRemainder } =
      addressParser.fromBytesWithRemainder(updatedRemainder!)

    const adminAddress = bytesToHex(unwrap(addressResult, 'Couldnt parse address'))

    const { result: percentageResult, remainder: percentageRemainder } =
      stringParser.fromBytesWithRemainder(addressRemainder!)

    const protocolFeeType = lowerCaseFirstLetter(
      unwrap(percentageResult, 'Couldnt parse percentage')
    )

    const { result, remainder } = u256Parser.fromBytesWithRemainder(percentageRemainder!)

    const protocolFee = BigInt(unwrap(result, 'Couldnt parse u256'))

    if (remainder!.length != 0) {
      throw new Error('There are remaining bytes left')
    }

    const config = {
      [structName]: {
        admin: adminAddress,
        protocolFee: { [protocolFeeType]: protocolFee }
      }
    }
    return config
  }

  async getFeeTiers() {
    const key = hash('fee_tiers')
    const stateRootHash = await this.service.getStateRootHash()
    const response = await this.client.nodeClient.getDictionaryItemBytesByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key
    )

    const bytes = new Uint8Array(
      String(response)
        .match(/.{1,2}/g)!
        .map((byte: string) => parseInt(byte, 16))
    )

    console.log(bytes)

    const stringParser = new CLStringBytesParser()
    const u32Parser = new CLU32BytesParser()
    const u128Parser = new CLU128BytesParser()

    const { remainder: stringRemainder } = stringParser.fromBytesWithRemainder(bytes)

    const { result: resul2, remainder: remainder2 } = u32Parser.fromBytesWithRemainder(
      stringRemainder!
    )
    const feeTierCount = BigInt(unwrap(resul2, 'Couldnt parse u32'))
    const feeTiers = []

    for (let i = 0; i < feeTierCount; i++) {
      const { remainder: feeTierBytes } = stringParser.fromBytesWithRemainder(remainder2!)

      const { result: feeTypeBytes, remainder: feeTypeRemainder } =
        stringParser.fromBytesWithRemainder(feeTierBytes!)

      const feeType = lowerCaseFirstLetter(unwrap(feeTypeBytes, 'Couldnt parse string'))

      const { result: feeBytes, remainder: feeRemainder } = u128Parser.fromBytesWithRemainder(
        feeTypeRemainder!
      )

      const fee = BigInt(unwrap(feeBytes, 'Couldnt parse u128'))

      const { result: tickSpacingBytes, remainder } = u32Parser.fromBytesWithRemainder(
        feeRemainder!
      )

      if (remainder!.length != 0) {
        throw new Error('There are remaining bytes left')
      }

      const tickSpacing = BigInt(unwrap(tickSpacingBytes, 'Couldnt parse u32'))

      const feeTier = { [feeType]: fee, tickSpacing }
      feeTiers.push(feeTier)
    }

    return feeTiers
  }

  async getPool() {
    const buffor: number[] = []
    // TODO: Add dynamic serialization for pool keys
    const poolKeyBytes = [
      7, 0, 0, 0, 80, 111, 111, 108, 75, 101, 121, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
      2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 7, 0, 0, 0, 70, 101, 101, 84, 105,
      101, 114, 10, 0, 0, 0, 80, 101, 114, 99, 101, 110, 116, 97, 103, 101, 1, 100, 10, 0, 0, 0
    ]
    buffor.push(...'pools'.split('').map(c => c.charCodeAt(0)))
    buffor.push('#'.charCodeAt(0))
    buffor.push(...'pools'.split('').map(c => c.charCodeAt(0)))
    buffor.push(...poolKeyBytes)

    const key = hash(new Uint8Array(buffor))

    const stateRootHash = await this.service.getStateRootHash()
    const response = await this.client.nodeClient.getDictionaryItemBytesByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key
    )

    const optionParser = new CLOptionBytesParser()
    const u256Parser = new CLU256BytesParser()
    const stringParser = new CLStringBytesParser()
    const i32Parser = new CLI32BytesParser()
    const u64Parser = new CLU64BytesParser()
    const addressParser = new CLAccountHashBytesParser()
    const boolParser = new CLBoolBytesParser()

    const bytes = new Uint8Array(
      String(response)
        .match(/.{1,2}/g)!
        .map((byte: string) => parseInt(byte, 16))
    )

    const expectedType = new CLOptionType(CLTypeBuilder.string())
    const { remainder: optionRemainder } = optionParser.fromBytesWithRemainder(bytes, expectedType)

    const { remainder: liquidityTypeRemainder } = stringParser.fromBytesWithRemainder(
      optionRemainder!
    )

    const { result: liquidityBytes, remainder: liquidityRemainder } =
      u256Parser.fromBytesWithRemainder(liquidityTypeRemainder!)

    const liquidity = BigInt(unwrap(liquidityBytes, 'Couldnt parse u256'))

    const { remainder: sqrtPriceTypeRemainder } = stringParser.fromBytesWithRemainder(
      liquidityRemainder!
    )

    const { result: sqrtPriceBytes, remainder: sqrtPriceRemainder } =
      u256Parser.fromBytesWithRemainder(sqrtPriceTypeRemainder!)

    const sqrtPrice = BigInt(unwrap(sqrtPriceBytes, 'Couldnt parse u256'))

    const { result: currentTickBytes, remainder: currentTickRemainder } =
      i32Parser.fromBytesWithRemainder(sqrtPriceRemainder!)

    const currentTickIndex = BigInt(unwrap(currentTickBytes, 'Couldnt parse i32'))

    const { remainder: feeGrowthGlobalXTypeRemainder } = stringParser.fromBytesWithRemainder(
      currentTickRemainder!
    )

    const { result: feeGrowthGlobalXBytes, remainder: feeGrowthGlobalXRemainder } =
      u256Parser.fromBytesWithRemainder(feeGrowthGlobalXTypeRemainder!)

    const feeGrowthGlobalX = BigInt(unwrap(feeGrowthGlobalXBytes, 'Couldnt parse u256'))

    const { remainder: feeGrowthGlobalYTypeRemainder } = stringParser.fromBytesWithRemainder(
      feeGrowthGlobalXRemainder!
    )

    const { result: feeGrowthGlobalYBytes, remainder: feeGrowthGlobalYRemainder } =
      u256Parser.fromBytesWithRemainder(feeGrowthGlobalYTypeRemainder!)

    const feeGrowthGlobalY = BigInt(unwrap(feeGrowthGlobalYBytes, 'Couldnt parse u256'))

    const { remainder: feeProtocolTokenXTypeRemainder } = stringParser.fromBytesWithRemainder(
      feeGrowthGlobalYRemainder!
    )

    const { result: feeProtocolTokenXBytes, remainder: feeProtocolTokenXRemainder } =
      u256Parser.fromBytesWithRemainder(feeProtocolTokenXTypeRemainder!)

    const feeProtocolTokenX = BigInt(unwrap(feeProtocolTokenXBytes, 'Couldnt parse u256'))

    const { remainder: feeProtocolTokenYTypeRemainder } = stringParser.fromBytesWithRemainder(
      feeProtocolTokenXRemainder!
    )

    const { result: feeProtocolTokenYBytes, remainder: feeProtocolTokenYRemainder } =
      u256Parser.fromBytesWithRemainder(feeProtocolTokenYTypeRemainder!)

    const feeProtocolTokenY = BigInt(unwrap(feeProtocolTokenYBytes, 'Couldnt parse u256'))

    const { result: startTimestampBytes, remainder: startTimestampRemainder } =
      u64Parser.fromBytesWithRemainder(feeProtocolTokenYRemainder!)

    const startTimestamp = BigInt(unwrap(startTimestampBytes, 'Couldnt parse u64'))

    const { result: lastTimestampBytes, remainder: lastTimestampRemainder } =
      u64Parser.fromBytesWithRemainder(startTimestampRemainder!)

    const lastTimestamp = BigInt(unwrap(lastTimestampBytes, 'Couldnt parse u64'))

    // One additional byte is left on the beggining of the bytes remainder
    const updatedRemainder = lastTimestampRemainder!.slice(1, lastTimestampRemainder!.length)

    const { result: adminAddressBytes, remainder: adminAddressRemainder } =
      addressParser.fromBytesWithRemainder(updatedRemainder!)

    const adminAddress = bytesToHex(unwrap(adminAddressBytes, 'Couldnt parse address'))

    const { result: oracleBytes, remainder } = boolParser.fromBytesWithRemainder(
      adminAddressRemainder!
    )

    const oracle = unwrap(oracleBytes, 'Couldnt parse bool')

    if (remainder!.length != 0) {
      throw new Error('There are remaining bytes left')
    }

    return {
      liquidity,
      sqrtPrice,
      currentTickIndex,
      feeGrowthGlobalX,
      feeGrowthGlobalY,
      feeProtocolTokenX,
      feeProtocolTokenY,
      startTimestamp,
      lastTimestamp,
      admin: adminAddress,
      oracle
    }
  }
}
