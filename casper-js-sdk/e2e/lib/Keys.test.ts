import * as fs from 'fs';
import * as Crypto from 'crypto';
import * as path from 'path';
import * as os from 'os';

import { expect } from 'chai';
import * as ed25519 from '@noble/ed25519';
import { decodeBase16, encodeBase16, encodeBase64 } from '../../src';
import { Ed25519, Secp256K1 } from '../../src/lib/Keys';
import { byteHash } from '../../src/lib/ByteConverters';

describe('Ed25519', () => {
  it('calculates the account hash', () => {
    const signKeyPair = Ed25519.new();
    // use lower case for node-rs
    const name = Buffer.from('ED25519'.toLowerCase());
    const sep = decodeBase16('00');
    const bytes = Buffer.concat([name, sep, signKeyPair.publicKey.value()]);
    const hash = byteHash(bytes);

    expect(Ed25519.accountHash(signKeyPair.publicKey.value())).deep.equal(hash);
  });

  it('should generate PEM file for Ed25519 correctly', () => {
    const naclKeyPair = Ed25519.new();
    const publicKeyInPem = naclKeyPair.exportPublicKeyInPem();
    const privateKeyInPem = naclKeyPair.exportPrivateKeyInPem();

    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'test-'));
    fs.writeFileSync(tempDir + '/public.pem', publicKeyInPem);
    fs.writeFileSync(tempDir + '/private.pem', privateKeyInPem);
    const signKeyPair2 = Ed25519.parseKeyFiles(
      tempDir + '/public.pem',
      tempDir + '/private.pem'
    );

    // expect nacl could import the generated PEM
    expect(encodeBase64(naclKeyPair.publicKey.value())).to.equal(
      encodeBase64(signKeyPair2.publicKey.value())
    );

    expect(encodeBase64(naclKeyPair.privateKey)).to.equal(
      encodeBase64(signKeyPair2.privateKey)
    );

    // import pem file to nodejs std library
    const pubKeyImported = Crypto.createPublicKey(publicKeyInPem);
    const priKeyImported = Crypto.createPrivateKey(privateKeyInPem);
    expect(pubKeyImported.asymmetricKeyType).to.equal('ed25519');

    // expect nodejs std lib export the same pem.
    const publicKeyInPemFromNode = pubKeyImported.export({
      type: 'spki',
      format: 'pem'
    });
    const privateKeyInPemFromNode = priKeyImported.export({
      type: 'pkcs8',
      format: 'pem'
    });
    expect(publicKeyInPemFromNode).to.equal(publicKeyInPem);
    expect(privateKeyInPemFromNode).to.equal(privateKeyInPem);

    // expect both of they generate the same signature
    const message = Buffer.from('hello world');
    const signatureByNode = Crypto.sign(null, message, priKeyImported);
    const signatureByNacl = naclKeyPair.sign(message);
    expect(encodeBase64(signatureByNode)).to.eq(encodeBase64(signatureByNacl));

    // expect both of they could verify by their own public key
    expect(Crypto.verify(null, message, pubKeyImported, signatureByNode)).to
      .true;
    expect(
      ed25519.sync.verify(
        signatureByNacl,
        message,
        naclKeyPair.publicKey.value()
      )
    ).to.true;
  });
});

describe('Secp256K1', () => {
  it('should generate PEM file for Secp256K1 correctly', () => {
    const signKeyPair = Secp256K1.new();

    // export key in pem to save
    const publicKeyInPem = signKeyPair.exportPublicKeyInPem();
    const privateKeyInPem = signKeyPair.exportPrivateKeyInPem();

    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'test-'));
    fs.writeFileSync(tempDir + '/public.pem', publicKeyInPem);
    fs.writeFileSync(tempDir + '/private.pem', privateKeyInPem);

    // expect importing keys from pem files works well
    expect(Secp256K1.parsePublicKeyFile(tempDir + '/public.pem')).to.deep.eq(
      signKeyPair.publicKey.value()
    );
    expect(Secp256K1.parsePrivateKeyFile(tempDir + '/private.pem')).to.deep.eq(
      signKeyPair.privateKey
    );

    const signKeyPair2 = Secp256K1.parseKeyFiles(
      tempDir + '/public.pem',
      tempDir + '/private.pem'
    );

    // expect parseKeyFiles could import files
    expect(encodeBase64(signKeyPair.publicKey.value())).to.equal(
      encodeBase64(signKeyPair2.publicKey.value())
    );
    expect(encodeBase64(signKeyPair.privateKey)).to.equal(
      encodeBase64(signKeyPair2.privateKey)
    );

    // import pem file to nodejs std library
    const ecdh = Crypto.createECDH('secp256k1');
    ecdh.setPrivateKey(signKeyPair.privateKey);
    expect(ecdh.getPublicKey('hex', 'compressed')).to.deep.equal(
      encodeBase16(signKeyPair.publicKey.value())
    );
  });
});
