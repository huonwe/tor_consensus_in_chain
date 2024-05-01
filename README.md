# tor_consensus_in_chain
## 使用 substrate 构建区块链，并获取 tor 共识信息，将其储存在链上。
## This project uses blockchain based on substrate, it will get the tor consensus from local tor node and update it to the chain.
# 编译指令
```bash
cd tor_consensus_in_chain
cargo build --release
./target/release/solochian-template-node
# get tor consensus
curl http://localhost:8080/update
```
