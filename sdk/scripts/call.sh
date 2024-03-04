# Query config
# casper-client get-dictionary-item \
#     --node-address http://195.201.174.222:7777 \
#     --state-root-hash 582cc03e088113f1d83f7665a1b929a0ec3942c73de9db2efa74a740f18fe158 \
#     --dictionary-name state \
#     --contract-hash hash-9195c4b7241c845bbc8bbe8801650fbd3c93d55e0c0145400edbdd5a7daa8a63 \
#     --dictionary-item-key 7071474438b622de882472abc92f9d7e1fd3456e19b46f0117fe607b9d819679

# casper-client get-dictionary-item \
#     --node-address http://195.201.174.222:7777 \
#     --state-root-hash e5b52844ed1fed3840a73658a41b23e6ed4daadfd503c4470754bef45e2584ef \
#     --dictionary-name state \
#     --contract-hash hash-0ecb4bf15c947eec96035dd41caf659144d7b219d2ab8cec69546835e05fdc88 \
#     --dictionary-item-key 21e7bb4efba167928b6af57c153f9facd4508b9a91a19b142eb0eb0ae573278c

# Query fee Tiers
# casper-client get-dictionary-item \
#     --node-address http://76.91.193.251:7777 \
#     --state-root-hash 798576816fd527b1d3488bf706a5e78474715e74ec6abb9530ddfa553a130735 \
#     --dictionary-name state \
#     --contract-hash hash-fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810 \
#     --dictionary-item-key 1b657452b4a00a04c8e90065b7e23840630ea759121af571b29245825195d140

# Query mapping in parent contract
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash 03377d6489cf5e09cc239c571a1d4de3e0ae1ad64ad0d107cb9cebb6dc3253e0 \
    --dictionary-name state \
    --contract-hash hash-f930178220e1956abacdb2a39a8597025a2cf0e0d38878d08f57d0f70a5ca67f \
    --dictionary-item-key 8993177b688dbcd454730d11f28d54508151536789928beb4deff08cc5a3e786

# Query nested mapping
casper-client get-dictionary-item \
    --node-address http://76.91.193.251:7777 \
    --state-root-hash 03377d6489cf5e09cc239c571a1d4de3e0ae1ad64ad0d107cb9cebb6dc3253e0 \
    --dictionary-name state \
    --contract-hash hash-f930178220e1956abacdb2a39a8597025a2cf0e0d38878d08f57d0f70a5ca67f \
    --dictionary-item-key 901d31919060f016ad1202d83e9398cc5962b3dad8f0f694d8bf6e79871a4f25
