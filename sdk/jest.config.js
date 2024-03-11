/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testTimeout: 300000,
  transform: {
    '^.+\\.test.ts?$': 'ts-jest'
  }
}
