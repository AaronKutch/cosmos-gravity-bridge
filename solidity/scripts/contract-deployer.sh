#!/bin/bash
npx ts-node \
contract-deployer.ts \
--cosmos-node="http://localhost:26657" \
--eth-node="http://localhost:8545" \
--eth-privkey="0xb1bab011e03a9862664706fc3bbaa1b16651528e5f0e7fbfcbfdd8be302a13e7" \
--contract=Gravity.json \
--test-mode=true \
--bnom-address="0x0F23c3f0C77582a5dB7fB3D61097B619982fb32f"