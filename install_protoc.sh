#!/bin/bash
set -e
set -x

VERSION=3.18.1

sudo apt-get -y install zip tree
curl -L https://github.com/protocolbuffers/protobuf/releases/download/v$VERSION/protoc-$VERSION-linux-x86_64.zip -o protoc.zip
unzip -o protoc.zip -d protoc

mkdir temp
pushd temp
  curl -L https://github.com/protocolbuffers/protobuf/releases/download/v$VERSION/protoc-$VERSION-linux-x86_64.zip -o protoc.zip
  unzip -o protoc.zip -d protoc
  sudo mv protoc/bin/* /usr/local/bin/
  sudo mv protoc/include/* /usr/local/include/
popd
rm -rf temp

tree /usr/local/include/google
