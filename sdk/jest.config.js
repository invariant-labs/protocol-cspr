/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testTimeout: 300000,
  transform: {
    '^.+\\.jsx?$': 'babel-jest'
  }
}
