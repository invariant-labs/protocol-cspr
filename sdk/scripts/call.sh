# # Query config
# casper-client get-dictionary-item \
#     --node-address http://195.201.174.222:7777 \
#     --state-root-hash 582cc03e088113f1d83f7665a1b929a0ec3942c73de9db2efa74a740f18fe158 \
#     --dictionary-name state \
#     --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
#     --dictionary-item-key 7071474438b622de882472abc92f9d7e1fd3456e19b46f0117fe607b9d819679

# # Query fee Tiers
# casper-client get-dictionary-item \
#     --node-address http://76.91.193.251:7777 \
#     --state-root-hash 798576816fd527b1d3488bf706a5e78474715e74ec6abb9530ddfa553a130735 \
#     --dictionary-name state \
#     --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
#     --dictionary-item-key 1b657452b4a00a04c8e90065b7e23840630ea759121af571b29245825195d140

# # Query nested mapping
# casper-client get-dictionary-item \
#     --node-address http://76.91.193.251:7777 \
#     --state-root-hash 03377d6489cf5e09cc239c571a1d4de3e0ae1ad64ad0d107cb9cebb6dc3253e0 \
#     --dictionary-name state \
#     --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
#     --dictionary-item-key 35018fd1c483d17c62fc396d1cc328d65efd6ea8e88eb4d3963cca1fb554702c

# Query Ticks
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash c668f105c664298a1d1820bacdf397c85f75922bccadbd851065ab34e7b31c55 \
    --dictionary-name state \
    --contract-hash hash-20f479456f71d612ee3c05c949e8faaec16c16a0af05a1d14dd0414be9978d2e \
    --dictionary-item-key 375d5382de938b330fc1ad91a5a5c6bb5d4190b8b9420abc6be5d290c91d6125

# Query tickmap chunk
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash c668f105c664298a1d1820bacdf397c85f75922bccadbd851065ab34e7b31c55 \
    --dictionary-name state \
    --contract-hash hash-20f479456f71d612ee3c05c949e8faaec16c16a0af05a1d14dd0414be9978d2e \
    --dictionary-item-key 41e0454aeba5f6c205372ea3c8f438504f8fcfce9cc039160fcfb6e4c3c6426c

# Query Position
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash c668f105c664298a1d1820bacdf397c85f75922bccadbd851065ab34e7b31c55 \
    --dictionary-name state \
    --contract-hash hash-20f479456f71d612ee3c05c949e8faaec16c16a0af05a1d14dd0414be9978d2e \
    --dictionary-item-key f7ba70e9d9249dba702e40dc471034a3c543c3579bf0fb8f6074fae43074f451

# Query Position length
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash c668f105c664298a1d1820bacdf397c85f75922bccadbd851065ab34e7b31c55 \
    --dictionary-name state \
    --contract-hash hash-20f479456f71d612ee3c05c949e8faaec16c16a0af05a1d14dd0414be9978d2e \
    --dictionary-item-key b2ac369f1c70360e4b3151167989e468ea0c80bd3c0f78bd9a8a720a1eca7953
