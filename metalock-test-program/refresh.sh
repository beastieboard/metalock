#!/bin/bash

cd `dirname $0`

set -e

cargo build-sbf
solana program deploy -u localhost ../target/deploy/metalock_test_program.so

