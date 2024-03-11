/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testTimeout: 30000,
  globals: {
    'ts-jest': {
      // Enable experimental VM modules
      useESM: true
    }
  }
}
