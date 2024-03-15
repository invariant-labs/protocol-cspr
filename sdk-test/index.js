import { initCasperClient, LOCAL_NODE_URL } from '@invariant-labs/cspr-sdk'

const main = async () => {
  initCasperClient(LOCAL_NODE_URL)
}

main()
