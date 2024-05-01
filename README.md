# tor_consensus_in_chain
## 使用 substrate 构建区块链，并获取 tor 共识信息，将其储存在链上。默认情况下，Tor节点的OR端口应当为7000，否则需要在程序源码中修改。
## This project uses blockchain based on substrate, it will get the tor consensus from local tor node and update it to the chain. The OR port of tor node should be 7000, or you will need to change it in code manually.
# 使用方法
```bash
cd tor_consensus_in_chain
cargo build --release
./target/release/solochian-template-node
# get tor consensus
curl http://localhost:8080/update
```
