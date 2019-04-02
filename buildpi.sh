#!/bin/bash

cargo build
ext=$?
if [[ $ext -ne 0 ]]; then
    exit $ext
fi

scp -i ~/.ssh/id_rsa_backup target/debug/tcp_rust root@52.43.105.190:~/justin