# Query config
casper-client get-dictionary-item \
    --node-address http://195.201.174.222:7777 \
    --state-root-hash 582cc03e088113f1d83f7665a1b929a0ec3942c73de9db2efa74a740f18fe158 \
    --dictionary-name state \
    --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
    --dictionary-item-key 7071474438b622de882472abc92f9d7e1fd3456e19b46f0117fe607b9d819679

# Query fee Tiers
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash 798576816fd527b1d3488bf706a5e78474715e74ec6abb9530ddfa553a130735 \
    --dictionary-name state \
    --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
    --dictionary-item-key 1b657452b4a00a04c8e90065b7e23840630ea759121af571b29245825195d140

# Query nested mapping
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash 03377d6489cf5e09cc239c571a1d4de3e0ae1ad64ad0d107cb9cebb6dc3253e0 \
    --dictionary-name state \
    --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
    --dictionary-item-key 35018fd1c483d17c62fc396d1cc328d65efd6ea8e88eb4d3963cca1fb554702c
