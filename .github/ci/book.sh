#!/bin/bash
## on push branch=main
## priority -100
## dedup dequeue
## cooldown 15m

set -euxo pipefail

make -C docs

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl get po -l app=website -o jsonpath={.items[0].metadata.name})

mkdir -p build
mv docs/book build/book
tar -C build -cf book.tar book
kubectl exec $POD -- mkdir -p /usr/share/nginx/html
kubectl cp book.tar $POD:/usr/share/nginx/html/
kubectl exec $POD -- find /usr/share/nginx/html
kubectl exec $POD -- tar -C /usr/share/nginx/html -xvf /usr/share/nginx/html/book.tar
