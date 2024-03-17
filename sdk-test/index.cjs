const { initCasperClient, LOCAL_NODE_URL } = require('@invariant-labs/cspr-sdk')

const main = async () => {
  initCasperClient(LOCAL_NODE_URL)
}

main()
